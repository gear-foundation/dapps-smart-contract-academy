use gear_wasm_builder::WasmBuilder;
use store_io::ProgramMetadata;

fn main() {
    WasmBuilder::with_meta(<ProgramMetadata as gmeta::Metadata>::repr())
        .exclude_features(vec!["binary-vendor"])
        .build();
}
