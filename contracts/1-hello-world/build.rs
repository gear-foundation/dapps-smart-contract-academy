use gear_wasm_builder::WasmBuilder;
use hello_world_io::ProgramMetadata;

fn main() {
    WasmBuilder::with_meta(<ProgramMetadata as gmeta::Metadata>::repr())
        .exclude_features(vec!["binary-vendor"])
        .build();
}
