//! 数据存储（版本索引和版本元数据）
//! 
//! ### 管理端文件存储结构
//! 
//! 1. public：存放更新包和索引文件的地方
//! 2. workspace：日常维护要更新的文件的地方
//! 5. config.toml：管理端的配置文件
//! 
//! ### public目录下的文件
//! 
//! public目录负责存储所有更新包文件，索引文件这些供大家下载的公共文件
//! 
//! 1. index.txt：索引文件，也叫版本号列表文件，会存储每个版本的元数据的信息
//! 2. index.internal.txt：同上，但是用于支持灰度发布
//! 3. combined.tar：合并包，所有合并后的更新包内容都会放到这个文件里，名字固定叫combined.tar
//! 4. 1.0.tar：用户创建的1.0版本更新包
//! 5. 1.1.tar：用户创建的1.1版本更新包
//! 6. 还有更多用户创建的更新包...
//! 
//! 合并包文件和普通用户创建更新包文件是个容器，一个文件里面可以容纳多个版本的数据。
//! 一般情况下，合并包会装多个版本的数据，而普通包只装一个版本的数据。
//! 在合并更新包时，所有的普通包内的内容会被全部挪动到合并包里面去

pub mod version_meta;
pub mod index_file;
pub mod version_meta_group;
