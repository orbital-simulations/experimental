/// Load WGSL source code from a file at compile time.
///
/// The loaded path is relative to the path of the file containing the macro call, in the same way
/// as [`include_str!`] operates.
///
/// ```ignore
/// fn main() {
///     let module: ShaderSource = include_wgsl!("shader.wgsl");
/// }
/// ```
#[macro_export]
macro_rules! include_wgsl {
    ($($token:tt)*) => {
        {
            $crate::resource_store::shader::ShaderSource::StaticFile($crate::resource_store::shader::StaticShaderFile {
                file_path: $($token)*,
                source: include_str!($($token)*).into(),
            })
        }
    };
}
