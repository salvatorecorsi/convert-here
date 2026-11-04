//! Anonymous native Windows HTTPS transport. Asynchronous WinHTTP retains its
//! callback context/read buffer until HANDLE_CLOSING, including cancellation.
use anyhow::{Context, Result, bail, ensure};
use std::{path::Path, sync::atomic::AtomicBool};

fn checked_url(value: &str, allow_loopback_http: bool) -> Result<url::Url> {
    let url = url::Url::parse(value).context("URL del distributore non valido")?;
    let loopback = matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "localhost"));
    ensure!(
        (url.scheme() == "https" || allow_loopback_http && loopback && url.scheme() == "http")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.fragment().is_none(),
        "Il download richiede un URL HTTPS senza credenziali"
    );
    Ok(url)
}

pub(super) fn download(
    url: &str,
    destination: &Path,
    expected: u64,
    cancel: &AtomicBool,
    progress: impl FnMut(u64),
) -> Result<()> {
    #[cfg(windows)]
    {
        native::download(url, destination, expected, cancel, progress, false)
    }
    #[cfg(not(windows))]
    {
        let _ = (url, destination, expected, cancel, progress);
        bail!("L’installazione automatica dei motori richiede Windows")
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        ffi::c_void,
        fs::OpenOptions,
        io::Write,
        ptr,
        sync::{Arc, Mutex, mpsc},
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{Foundation::ERROR_IO_PENDING, Networking::WinHttp::*};

    enum Event {
        Sent,
        Headers,
        Read(usize),
        Error(u32),
    }
    struct CallbackState {
        sender: mpsc::Sender<Event>,
        // The allocation lives with the raw Arc until the final closing callback.
        buffer: Mutex<Vec<u8>>,
    }
    struct Handle(*mut c_void);
    impl Handle {
        fn new(value: *mut c_void) -> Result<Self> {
            ensure!(
                !value.is_null(),
                "Impossibile aprire la connessione: {}",
                std::io::Error::last_os_error()
            );
            Ok(Self(value))
        }
    }
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                WinHttpCloseHandle(self.0);
            }
        }
    }

    unsafe extern "system" fn callback(
        _handle: *mut c_void,
        context: usize,
        status: u32,
        information: *mut c_void,
        length: u32,
    ) {
        if context == 0 {
            return;
        }
        if status == WINHTTP_CALLBACK_STATUS_HANDLE_CLOSING {
            drop(unsafe { Arc::from_raw(context as *const CallbackState) });
            return;
        }
        let state = unsafe { &*(context as *const CallbackState) };
        let event = match status {
            WINHTTP_CALLBACK_STATUS_SENDREQUEST_COMPLETE => Some(Event::Sent),
            WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE => Some(Event::Headers),
            WINHTTP_CALLBACK_STATUS_READ_COMPLETE => Some(Event::Read(length as usize)),
            WINHTTP_CALLBACK_STATUS_REQUEST_ERROR if !information.is_null() => {
                let error = unsafe { &*(information as *const WINHTTP_ASYNC_RESULT) };
                Some(Event::Error(error.dwError))
            }
            _ => None,
        };
        if let Some(event) = event {
            let _ = state.sender.send(event);
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }
    fn option(handle: &Handle, name: u32, value: u32) -> Result<()> {
        ensure!(
            unsafe { WinHttpSetOption(handle.0, name, (&value as *const u32).cast(), 4) } != 0,
            "Opzione HTTPS non disponibile: {}",
            std::io::Error::last_os_error()
        );
        Ok(())
    }
    fn initiated(success: i32) -> Result<()> {
        if success == 0 {
            let error = std::io::Error::last_os_error();
            ensure!(
                error.raw_os_error() == Some(ERROR_IO_PENDING as i32),
                "Richiesta HTTP non riuscita: {error}"
            );
        }
        Ok(())
    }
    fn wait(receiver: &mpsc::Receiver<Event>, cancel: &AtomicBool) -> Result<Event> {
        let start = Instant::now();
        loop {
            super::super::cancelled(cancel)?;
            ensure!(
                start.elapsed() < Duration::from_secs(60),
                "Il distributore non risponde"
            );
            match receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(Event::Error(code)) => bail!("Download interrotto da Windows: errore {code}"),
                Ok(event) => return Ok(event),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => bail!("Connessione HTTP interrotta"),
            }
        }
    }
    fn header(request: &Handle, name: u32) -> Option<String> {
        let mut text = vec![0u16; 8192];
        let mut length = (text.len() * 2) as u32;
        if unsafe {
            WinHttpQueryHeaders(
                request.0,
                name,
                ptr::null(),
                text.as_mut_ptr().cast(),
                &mut length,
                ptr::null_mut(),
            )
        } == 0
        {
            return None;
        }
        let length = text
            .iter()
            .position(|value| *value == 0)
            .unwrap_or((length / 2) as usize);
        Some(String::from_utf16_lossy(&text[..length]))
    }

    pub(super) fn download(
        source: &str,
        destination: &Path,
        expected: u64,
        cancel: &AtomicBool,
        mut progress: impl FnMut(u64),
        allow_loopback_http: bool,
    ) -> Result<()> {
        let mut url = checked_url(source, allow_loopback_http)?;
        let agent = wide(concat!(
            "ConvertHere/",
            env!("CARGO_PKG_VERSION"),
            " (runtime installer)"
        ));
        let access = if allow_loopback_http {
            WINHTTP_ACCESS_TYPE_NO_PROXY
        } else {
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY
        };
        let session = Handle::new(unsafe {
            WinHttpOpen(
                agent.as_ptr(),
                access,
                ptr::null(),
                ptr::null(),
                WINHTTP_FLAG_ASYNC,
            )
        })?;
        option(
            &session,
            WINHTTP_OPTION_SECURE_PROTOCOLS,
            WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2 | WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3,
        )
        .or_else(|_| {
            // Older Windows 10 builds do not recognize the TLS 1.3 flag.
            option(
                &session,
                WINHTTP_OPTION_SECURE_PROTOCOLS,
                WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2,
            )
        })?;
        ensure!(
            unsafe { WinHttpSetTimeouts(session.0, 10_000, 10_000, 10_000, 15_000) } != 0,
            "Timeout HTTP non configurabile"
        );
        let verb = wide("GET");
        for _ in 0..=5 {
            super::super::cancelled(cancel)?;
            let host = wide(url.host_str().context("Distributore non disponibile")?);
            let connection = Handle::new(unsafe {
                WinHttpConnect(
                    session.0,
                    host.as_ptr(),
                    url.port_or_known_default().unwrap_or(443),
                    0,
                )
            })?;
            let object = wide(&format!(
                "{}{}",
                url.path(),
                url.query()
                    .map(|query| format!("?{query}"))
                    .unwrap_or_default()
            ));
            let secure = if url.scheme() == "https" {
                WINHTTP_FLAG_SECURE
            } else {
                0
            };
            let request = Handle::new(unsafe {
                WinHttpOpenRequest(
                    connection.0,
                    verb.as_ptr(),
                    object.as_ptr(),
                    ptr::null(),
                    ptr::null(),
                    ptr::null(),
                    secure,
                )
            })?;
            option(
                &request,
                WINHTTP_OPTION_DISABLE_FEATURE,
                WINHTTP_DISABLE_REDIRECTS
                    | WINHTTP_DISABLE_COOKIES
                    | WINHTTP_DISABLE_AUTHENTICATION,
            )?;
            let (sender, receiver) = mpsc::channel();
            let state = Arc::new(CallbackState {
                sender,
                buffer: Mutex::new(vec![0u8; 65536]),
            });
            let flags = WINHTTP_CALLBACK_FLAG_SENDREQUEST_COMPLETE
                | WINHTTP_CALLBACK_FLAG_HEADERS_AVAILABLE
                | WINHTTP_CALLBACK_FLAG_READ_COMPLETE
                | WINHTTP_CALLBACK_FLAG_REQUEST_ERROR
                | WINHTTP_CALLBACK_STATUS_HANDLE_CLOSING;
            let previous = unsafe { WinHttpSetStatusCallback(request.0, Some(callback), flags, 0) };
            ensure!(
                previous.map(|function| function as usize) != Some(usize::MAX),
                "Callback HTTP non disponibile"
            );
            let context = Arc::into_raw(state.clone()) as usize;
            if unsafe {
                WinHttpSetOption(
                    request.0,
                    WINHTTP_OPTION_CONTEXT_VALUE,
                    (&context as *const usize).cast(),
                    size_of::<usize>() as u32,
                )
            } == 0
            {
                drop(unsafe { Arc::from_raw(context as *const CallbackState) });
                bail!("Contesto HTTP non disponibile");
            }
            initiated(unsafe {
                WinHttpSendRequest(request.0, ptr::null(), 0, ptr::null(), 0, 0, context)
            })?;
            ensure!(
                matches!(wait(&receiver, cancel)?, Event::Sent),
                "Risposta HTTP inattesa"
            );
            initiated(unsafe { WinHttpReceiveResponse(request.0, ptr::null_mut()) })?;
            ensure!(
                matches!(wait(&receiver, cancel)?, Event::Headers),
                "Header HTTP non disponibili"
            );
            let status: u16 = header(&request, WINHTTP_QUERY_STATUS_CODE)
                .context("Stato HTTP mancante")?
                .parse()?;
            if matches!(status, 301 | 302 | 303 | 307 | 308) {
                let location = header(&request, WINHTTP_QUERY_LOCATION)
                    .context("Redirect privo di destinazione")?;
                url = checked_url(url.join(&location)?.as_str(), allow_loopback_http)?;
                continue;
            }
            ensure!(status == 200, "Il distributore ha restituito HTTP {status}");
            if let Some(length) = header(&request, WINHTTP_QUERY_CONTENT_LENGTH) {
                ensure!(
                    length.parse::<u64>()? == expected,
                    "Dimensione HTTP diversa dall’archivio previsto"
                );
            }
            let mut output = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(destination)?;
            let mut downloaded = 0u64;
            loop {
                super::super::cancelled(cancel)?;
                let pointer = state
                    .buffer
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Buffer HTTP non disponibile"))?
                    .as_mut_ptr();
                initiated(unsafe {
                    WinHttpReadData(request.0, pointer.cast(), 65536, ptr::null_mut())
                })?;
                let Event::Read(count) = wait(&receiver, cancel)? else {
                    bail!("Dati HTTP inattesi");
                };
                if count == 0 {
                    break;
                }
                downloaded = downloaded
                    .checked_add(count as u64)
                    .context("Download troppo grande")?;
                ensure!(
                    downloaded <= expected && count <= 65536,
                    "Download più grande dell’archivio previsto"
                );
                let buffer = state
                    .buffer
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Buffer HTTP non disponibile"))?;
                output.write_all(&buffer[..count])?;
                progress(downloaded);
            }
            ensure!(downloaded == expected, "Download incompleto");
            output.sync_all()?;
            return Ok(());
        }
        bail!("Troppi redirect dal distributore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_urls_require_https_and_never_accept_embedded_credentials() {
        for value in [
            "http://github.com/file",
            "ftp://example.com/a",
            "https://user:password@example.com/a",
            "https://example.com/a#fragment",
        ] {
            assert!(checked_url(value, false).is_err());
        }
        assert!(
            checked_url(
                "https://release-assets.githubusercontent.com/a?token=release",
                false
            )
            .is_ok()
        );
    }

    #[cfg(windows)]
    fn request(stream: &mut std::net::TcpStream) -> String {
        use std::io::Read;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        let mut header = Vec::new();
        while !header.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            assert_eq!(stream.read(&mut byte).unwrap(), 1);
            header.push(byte[0]);
            assert!(header.len() < 16384);
        }
        String::from_utf8(header).unwrap()
    }

    #[cfg(windows)]
    #[test]
    fn runtime_install_native_http_fixture_follows_redirect_without_credentials_or_cookies() {
        use std::{io::Write, net::TcpListener};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let body = b"inert archive fixture, never executed";
        let server = std::thread::spawn(move || {
            for index in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let header = request(&mut stream);
                assert!(!header.to_ascii_lowercase().contains("\r\ncookie:"));
                assert!(!header.to_ascii_lowercase().contains("\r\nauthorization:"));
                if index == 0 {
                    write!(stream,"HTTP/1.1 302 Found\r\nLocation: http://{address}/payload\r\nSet-Cookie: unwanted=1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                } else {
                    assert!(header.starts_with("GET /payload "));
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .unwrap();
                    stream.write_all(body).unwrap();
                }
            }
        });
        let temporary = tempfile::tempdir().unwrap();
        let destination = temporary.path().join("download.zip");
        let mut bytes = 0;
        native::download(
            &format!("http://{address}/redirect"),
            &destination,
            body.len() as u64,
            &AtomicBool::new(false),
            |count| bytes = count,
            true,
        )
        .unwrap();
        server.join().unwrap();
        assert_eq!(std::fs::read(destination).unwrap(), body);
        assert_eq!(bytes, body.len() as u64);
    }

    #[cfg(windows)]
    #[test]
    fn runtime_install_native_http_cancels_an_inflight_response() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            sync::{Arc, atomic::Ordering, mpsc},
            time::Duration,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (ready, waiting) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            request(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4096\r\nConnection: close\r\n\r\n")
                .unwrap();
            stream.flush().unwrap();
            ready.send(()).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut byte = [0];
            match stream.read(&mut byte) {
                Ok(count) => assert_eq!(count, 0, "Cancellation must close the native request"),
                Err(error) => assert!(matches!(
                    error.kind(),
                    std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
                )),
            }
        });
        let cancel = Arc::new(AtomicBool::new(false));
        let controller = cancel.clone();
        let cancel_thread = std::thread::spawn(move || {
            waiting.recv_timeout(Duration::from_secs(3)).unwrap();
            std::thread::sleep(Duration::from_millis(100));
            controller.store(true, Ordering::Relaxed);
        });
        let temporary = tempfile::tempdir().unwrap();
        let result = native::download(
            &format!("http://{address}/slow"),
            &temporary.path().join("partial.zip"),
            4096,
            &cancel,
            |_| {},
            true,
        );
        assert!(result.is_err());
        assert!(cancel.load(Ordering::Relaxed));
        cancel_thread.join().unwrap();
        server.join().unwrap();
    }
}
