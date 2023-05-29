use escrow_io::ProgramMetadata;
use gear_wasm_builder::WasmBuilder;

fn main() {
    WasmBuilder::with_meta(<ProgramMetadata as gmeta::Metadata>::repr())
        .exclude_features(vec!["binary-vendor"])
        .build();
}
