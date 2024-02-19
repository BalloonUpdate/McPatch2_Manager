use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::data::version_meta::FileChange;
use crate::data::version_meta_group::VersionMetaGroup;
use crate::utility::counted_write::CountedWrite;

pub struct MetadataLocation {
    pub offset: u64,
    pub length: u32,
}

/// 代表一个tar包写入器，用于生成tar格式的更新包
pub struct TarWriter {
    builder: tar::Builder<CountedWrite<std::fs::File>>,
    addresses: HashMap<String, u64>,
    finished: bool,
}

impl TarWriter {
    /// 创建一个tar包写入器，并将数据写入到`version`文件中
    pub fn new(version: impl AsRef<Path>) -> Self {
        let open = std::fs::File::options().create(true).truncate(true).write(true).open(version).unwrap();

        Self {
            builder: tar::Builder::new(CountedWrite::new(open)), 
            addresses: HashMap::new(),
            finished: false,
        }
    }

    /// 往tar包里添加一个文件
    pub fn write_file(&mut self, data: impl Read, len: u64, path: &str, version: &str) {
        assert!(!self.finished, "TarWriter has already closed");

        // 记录当前指针位置
        let key = format!("{}_{}", path, version);
        self.addresses.insert(key, self.builder.get_ref().count() + 512);

        // 写入tar包中
        let mut header = tar::Header::new_gnu();
        header.set_size(len);
        self.builder.append_data(&mut header, path, data).unwrap();
    }

    /// 完成tar包的创建，并返回元数据的偏移值和长度
    pub fn finish(mut self, mut meta_group: VersionMetaGroup) -> MetadataLocation {
        assert!(!self.finished, "TarWriter has already closed");

        // 更新meta中的数据偏移值
        for meta in &mut meta_group {
            for change in meta.changes.iter_mut() {
                if let FileChange::UpdateFile { path, offset, .. } = change {
                    // 合并文件时，中间版本中的文件数据为了节省空间，是不存储的，也就是不更新offset
                    // 正常情况下客户端也不会去这个数据，如果读取了，那么不是数据受损就是客户端有问题
                    let key = format!("{}_{}", path, &meta.label);
                    match self.addresses.get(&key) {
                        Some(addr) => *offset = *addr,
                        None => (),
                    }
                }
            }
        }

        // 写入元数据
        let metadata_offset = self.builder.get_ref().count();
        let file_content = meta_group.serialize();
        let file_content = file_content.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_size(file_content.len() as u64);
        self.builder.append_data(&mut header, "metadata.txt", std::io::Cursor::new(&file_content)).unwrap();

        // 写入完毕
        self.builder.finish().unwrap();

        self.finished = true;

        MetadataLocation {
            offset: metadata_offset + 512,
            length: file_content.len() as u32,
        }
    }
}

impl Drop for TarWriter {
    fn drop(&mut self) {
        assert!(self.finished, "TarWriter has not closed yet");
    }
}