use super::*;

struct FotoTemporaria(std::path::PathBuf);

impl FotoTemporaria {
    fn nova() -> Self {
        let nome = format!(
            "ies-bdd60-foto-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("relógio")
                .as_nanos()
        );
        let pasta = std::env::temp_dir().join(nome);
        std::fs::create_dir(&pasta).expect("diretório da foto");
        Self(pasta)
    }
}

impl Drop for FotoTemporaria {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// BDD-60-02: --processo + --versao deve servir a revisão pedida,
// fotografá-la e não substituir edição local divergente na releitura.
#[test]
fn bdd_60_02_cli_versao_e_foto_no_processo_informado() {
    let destino = FotoTemporaria::nova();
    let dir = destino.0.to_str().expect("destino UTF-8");
    let args = [
        "--json",
        "documento",
        DOC_A,
        "--processo",
        NUP_A,
        "--versao",
        "1",
        "--saida",
        dir,
        "--sem-cache",
    ];
    let (primeira, trilha) = executar(true, &args);
    assert_eq!(
        primeira.status.code(),
        Some(0),
        "stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&primeira.stderr)
    );
    let valor = json(&primeira);
    let doc = &valor["documentos"][0];
    assert_eq!(doc["numero"], DOC_A);
    assert_eq!(doc["processo"]["nup"], NUP_A);
    assert_eq!(doc["versao"], 1);
    assert_eq!(doc["arquivo"]["nome"], "0011519.v1.html");
    assert_eq!(doc["arquivo"]["gravado"], true);
    assert_eq!(valor["requisicoes"].as_u64(), Some(trilha.len() as u64));
    let foto = destino.0.join("0011519.v1.html");
    let html = std::fs::read_to_string(&foto).expect("foto da versão histórica");
    assert!(html.contains("primeiro processo"), "{html}");
    assert!(!html.contains("segundo processo"), "{html}");
    assert!(!destino.0.join("0011519.v2.html").exists());

    // A divergência é só no corpo: título e rodapé mantêm identidade
    // e versão coerentes, exigindo comparação com a leitura do servidor.
    let editado = html.replacen(
        "Conteudo exclusivo do primeiro processo",
        "Conteudo editado localmente",
        1,
    );
    assert_ne!(
        editado, html,
        "a revisão simulada não contém o corpo esperado"
    );
    std::fs::write(&foto, &editado).expect("edição de teste");
    let (segunda, trilha) = executar(true, &args);
    assert_eq!(
        segunda.status.code(),
        Some(1),
        "foto divergente não pode ser substituída: stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&segunda.stderr)
    );
    let _erro = json(&segunda);
    assert_eq!(std::fs::read_to_string(&foto).unwrap(), editado);
    assert!(!destino.0.join("0011519.v2.html").exists());
}
