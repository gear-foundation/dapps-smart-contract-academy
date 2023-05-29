use battle_io::BattleMetadata;
use gear_wasm_builder::WasmBuilder;

fn main() {
    WasmBuilder::with_meta(<BattleMetadata as gmeta::Metadata>::repr())
        .exclude_features(vec!["binary-vendor"])
        .build();
}
