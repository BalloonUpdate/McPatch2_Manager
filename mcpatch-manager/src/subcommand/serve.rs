//! 运行内置服务端，使用私有协议

use std::io::ErrorKind;
use std::net::TcpListener;
use std::ops::Range;
use std::time::SystemTime;

use mcpatch_shared::utility::partial_read::PartialAsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::AppContext;

pub fn do_serve(_ctx: &AppContext) -> i32 {
    println!("启动内置服务端");

    let host = "0.0.0.0";
    let port = "6700";

    let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
    println!("listening on {}:{}", host, port);

    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    for stream in listener.incoming() {
        println!("aaaaaaa");
        let stream = stream.unwrap();
        let ctx = _ctx.clone();
        runtime.spawn(async move { serve_loop(stream, ctx).await });
    }

    0
}

async fn serve_loop(stream: std::net::TcpStream, ctx: AppContext) {
    let mut stream = TcpStream::from_std(stream).unwrap();

    async fn inner(mut stream: &mut TcpStream, ctx: &AppContext, info: &mut Option<(String, Range<u64>)>) -> std::io::Result<()> {
        // 接收文件路径
        let mut path = String::with_capacity(1024);
        receive_data(&mut stream).await?.read_to_string(&mut path).await?;

        let start = stream.read_u64_le().await?;
        let mut end = stream.read_u64_le().await?;

        *info = Some((path.to_owned(), start..end));

        let path = ctx.public_dir.join(path);

        assert!(start <= end, "the end is {} and the start is {}", end, start);

        // 检查文件大小
        let len = match tokio::fs::metadata(&path).await {
            Ok(meta) => {
                // 请求的范围不对，返回-2
                if end > meta.len() {
                    stream.write_all(&(-2i64).to_le_bytes()).await?;
                    return Ok(());
                }
                meta.len()
            },
            Err(_) => {
                // 文件没有找到，返回-1
                stream.write_all(&(-1i64).to_le_bytes()).await?;
                return Ok(());
            },
        };

        // 如果不指定范围就发送整个文件
        if start == 0 && end == 0 {
            end = len as u64;
        }

        let mut remains = end - start;

        // 文件已经找到，发送文件大小
        stream.write_all(&(remains as i64).to_le_bytes()).await?;

        // 传输文件内容
        let mut file = tokio::fs::File::open(path).await?;
        file.seek(std::io::SeekFrom::Start(start)).await?;

        while remains > 0 {
            let mut buf = [0u8; 16 * 1024];
            let limit = buf.len().min(remains as usize);
            let read = file.read(&mut buf[0..limit]).await?;
            
            stream.write_all(&buf[0..read]).await?;

            remains -= read as u64;
        }

        Ok(())
    }

    loop {
        let mut info = Option::<(String, Range<u64>)>::None;

        let start = SystemTime::now();
        let result = inner(&mut stream, &ctx, &mut info).await;
        let time = SystemTime::now().duration_since(start).unwrap();

        if let Some(info) = info {
            println!("{} -- {} {}-{} ({}ms)", stream.peer_addr().unwrap(), info.0, info.1.start, info.1.end, time.as_micros());
        }
        
        match result {
            Ok(_) => {},
            Err(e) => {
                if e.kind() != ErrorKind::UnexpectedEof {
                    Result::<(), _>::Err(e).unwrap();
                }

                break;
            },
        }
    }
}

async fn _send_data(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    stream.write_u64_le(data.len() as u64).await?;
    stream.write_all(data).await?;

    Ok(())
}

async fn receive_data<'a>(stream: &'a mut TcpStream) -> std::io::Result<PartialAsyncRead<'a, TcpStream>> {
    let len = stream.read_u64_le().await?;

    Ok(PartialAsyncRead::new(stream, len))
}