//! #60 BDD-60-05/06: testes CLI documentais contra anti-IES e SQLite.
//! Suporte TLS e estado efemero em `support/documento_interprocessos.rs`.
#![cfg(unix)]
#[path = "support/documento_interprocessos.rs"]
mod apoio;
use apoio::{
    DOC_A, DOC_B, NUP_A, NUP_B, executar, executar_cancelado, executar_numero_repetido,
    executar_revogado_ao_visualizar, executar_sem_src, json,
};
use serde_json::Value;

#[test]
fn bdd_60_06_lote_interprocessos_preserva_identidades_e_conteudos() {
    let (saida, trilha) = executar(true, &["--json", "documento", DOC_A, DOC_B, "--sem-cache"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = json(&saida);
    let documentos = valor["documentos"].as_array().expect("lista de documentos");
    assert_eq!(documentos.len(), 2, "{valor}");
    for (doc, numero, nup, exclusivo, alheio) in [
        (
            &documentos[0],
            DOC_A,
            NUP_A,
            "primeiro processo",
            "segundo processo",
        ),
        (
            &documentos[1],
            DOC_B,
            NUP_B,
            "segundo processo",
            "primeiro processo",
        ),
    ] {
        assert_eq!(doc["numero"], numero);
        assert_eq!(doc["processo"]["nup"], nup);
        assert!(doc["texto"].as_str().unwrap().contains(exclusivo), "{doc}");
        assert!(!doc["texto"].as_str().unwrap().contains(alheio), "{doc}");
    }
    assert_eq!(valor["requisicoes"].as_u64(), Some(trilha.len() as u64));
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "não há escrita, apenas navegação permitida: {trilha:?}"
    );
}

#[test]
fn bdd_60_06_lote_nao_aceita_documento_sem_identidade_comprovada() {
    let (saida, trilha) = executar(false, &["--json", "documento", DOC_A, DOC_B, "--sem-cache"]);
    assert_ne!(
        saida.status.code(),
        Some(0),
        "não pode usar o conteúdo de A como B"
    );
    let valor = json(&saida);
    assert_eq!(valor["documento"], DOC_B, "{valor}");
    assert_eq!(valor["documentos_lidos_antes"], 1, "{valor}");
    assert!(
        valor.get("documentos").is_none(),
        "não publicar lote parcial: {valor}"
    );
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "a falha não autoriza escrita: {trilha:?}"
    );
}

#[test]
fn bdd_60_05_cli_recusa_documento_de_outro_processo() {
    let (saida, trilha) = executar(
        true,
        &[
            "--json",
            "documento",
            DOC_B,
            "--processo",
            NUP_A,
            "--sem-cache",
        ],
    );
    assert_eq!(
        saida.status.code(),
        Some(1),
        "recusa de identidade, não falha genérica de sessão: stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = json(&saida);
    assert_eq!(valor["codigo"], "fonte_invalida", "{valor}");
    // ErroHtml::ausente da localização tipada pelo contexto --processo;
    // preservar a causa já publicada, sem criar novo subcódigo.
    assert_eq!(
        valor["subcodigo"], "numero_sei_ausente_da_arvore_do_processo",
        "{valor}"
    );
    assert_eq!(valor["documento"], DOC_B, "{valor}");
    assert_eq!(valor["documentos_lidos_antes"], 0, "{valor}");
    assert!(valor.get("documentos").is_none(), "{valor}");
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "a recusa não autoriza escrita: {trilha:?}"
    );
}

#[test]
fn pesquisa_rapida_de_documento_confere_id_antes_de_servir_conteudo() {
    let (saida, trilha) = executar(true, &["--json", "documento", DOC_A, "--sem-cache"]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = json(&saida);
    let docs = valor["documentos"].as_array().expect("lote de documentos");
    assert_eq!(docs.len(), 1, "{valor}");
    assert_eq!(docs[0]["numero"], DOC_A);
    assert_eq!(docs[0]["processo"]["nup"], NUP_A);
    assert!(
        docs[0]["texto"]
            .as_str()
            .unwrap()
            .contains("primeiro processo")
    );
    assert_eq!(valor["requisicoes"].as_u64(), Some(trilha.len() as u64));
    assert!(
        trilha.iter().any(|p| p == "POST protocolo_pesquisa_rapida"),
        "exercitar pesquisa por número, não o cache: {trilha:?}"
    );
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "somente navegação de leitura: {trilha:?}"
    );
}

#[test]
fn bdd_60_04_cli_nao_inventa_propriedades_sem_link_na_barra() {
    // No mesmo SQLite do processo A, a barra não oferece cadastro.
    // O BDD exige distinguir propriedade não observada de cadastro vazio.
    let (saida, trilha) = executar(
        true,
        &[
            "--json",
            "documento",
            DOC_A,
            "--processo",
            NUP_A,
            "--com-propriedades",
            "--sem-cache",
        ],
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = json(&saida);
    let documento = &valor["documentos"][0];
    assert_eq!(documento["numero"], DOC_A);
    assert_eq!(documento["propriedades"], Value::Null, "{documento}");
    assert_eq!(
        documento["propriedades_motivo"], "nao_publicadas_na_barra",
        "{documento}"
    );
    assert_eq!(valor["requisicoes"].as_u64(), Some(trilha.len() as u64));
    assert!(
        !trilha
            .iter()
            .any(|p| p.contains("documento_alterar") || p.contains("documento_consultar")),
        "sem link publicado, nenhum GET de cadastro: {trilha:?}"
    );
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "a leitura não escreve: {trilha:?}"
    );
}

#[test]
fn bdd_60_05_documento_sem_src_recusa_e_oculta_somente_quando_solicitado() {
    // Dado um documento existente na árvore de NUP_A sem src publicado,
    // quando leio esse número sem opção de ocultação, então a CLI recusa
    // a ausência observada e não usa o corpo legível de DOC_A.
    const SEM_SRC: &str = "1000001";
    let (saida, trilha) = executar_sem_src(&[
        "--json",
        "documento",
        SEM_SRC,
        "--processo",
        NUP_A,
        "--sem-cache",
    ]);
    assert_eq!(
        saida.status.code(),
        Some(1),
        "deve recusar o conteúdo não publicado: stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let recusado = json(&saida);
    assert_eq!(recusado["documento"], SEM_SRC, "{recusado}");
    assert_eq!(
        recusado["subcodigo"], "conteudo_nao_publicado_na_arvore",
        "{recusado}"
    );
    assert!(recusado.get("documentos").is_none(), "{recusado}");
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "a recusa não provoca escrita: {trilha:?}"
    );

    // Com --ocultar-inacessiveis, o mesmo SQLite tem uma leitura válida.
    // Só o documento com src comprovado é apresentado, sem fallback.
    let (saida, trilha) = executar_sem_src(&[
        "--json",
        "documento",
        DOC_A,
        SEM_SRC,
        "--processo",
        NUP_A,
        "--ocultar-inacessiveis",
        "--sem-cache",
    ]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = json(&saida);
    let documentos = valor["documentos"].as_array().expect("lista");
    assert_eq!(documentos.len(), 1, "{valor}");
    assert_eq!(documentos[0]["numero"], DOC_A);
    assert_eq!(documentos[0]["processo"]["nup"], NUP_A);
    assert_eq!(valor["documentos_omitidos"], 1, "{valor}");
    assert_eq!(valor["requisicoes"].as_u64(), Some(trilha.len() as u64));
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "somente leitura no transporte: {trilha:?}"
    );
}

// BDD-60-05: cancelar no SQLite deve retirar a oferta de conteúdo
// da árvore sem permitir que --ocultar-inacessiveis masque cancelamento.
#[test]
fn bdd_60_05_cli_cancelado_nao_reaproveita_corpo_nem_e_ocultado() {
    for ocultar in [false, true] {
        let mut args = vec![
            "--json",
            "documento",
            DOC_A,
            "--processo",
            NUP_A,
            "--sem-cache",
        ];
        if ocultar {
            args.push("--ocultar-inacessiveis");
        }
        let (saida, trilha) = executar_cancelado(&args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "cancelamento não é documento legível nem omissão: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let erro = json(&saida);
        assert_eq!(
            erro["subcodigo"], "documento_cancelado_sem_conteudo_na_arvore",
            "{erro}"
        );
        assert_eq!(erro["documento"], DOC_A, "{erro}");
        assert!(erro.get("documentos").is_none(), "{erro}");
        assert!(
            !trilha
                .iter()
                .any(|pedido| pedido == "GET documento_visualizar"),
            "sem link não se requisita conteúdo cancelado: {trilha:?}"
        );
        assert!(
            trilha
                .iter()
                .all(|pedido| pedido.starts_with("GET ")
                    || pedido == "POST protocolo_pesquisa_rapida"),
            "sem POST mutador ou retry: {trilha:?}"
        );
    }
}

// BDD-60-01: o contexto NUP explícito deve selecionar o nó e o conteúdo
// do mesmo documento. O teste executa o binário real via TLS/anti-IES/SQLite,
// não apenas o parser de uma árvore sintética isolada.
#[test]
fn bdd_60_01_cli_leitura_com_processo_preserva_identidade_e_corpo() {
    let (saida, trilha) = executar(
        true,
        &[
            "--json",
            "documento",
            DOC_A,
            "--processo",
            NUP_A,
            "--sem-cache",
        ],
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = json(&saida);
    let documentos = valor["documentos"].as_array().expect("lote de documentos");
    assert_eq!(documentos.len(), 1, "{valor}");
    let documento = &documentos[0];
    assert_eq!(documento["numero"], DOC_A);
    assert_eq!(documento["id_documento"], "42");
    assert_eq!(documento["processo"]["nup"], NUP_A);
    assert_eq!(documento["processo"]["id"], "7");
    assert_eq!(documento["fonte"], "documento_visualizar");
    let corpo = documento["texto"].as_str().expect("corpo do documento");
    assert!(corpo.contains("primeiro processo"), "{documento}");
    assert!(!corpo.contains("segundo processo"), "{documento}");
    assert_eq!(valor["requisicoes"].as_u64(), Some(trilha.len() as u64));
    assert!(
        trilha
            .iter()
            .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
        "navegação sem escrita: {trilha:?}"
    );
}

#[path = "aies_documento_interprocessos_cli/fotos.rs"]
mod fotos;

#[path = "aies_documento_interprocessos_cli/titulo_trocado.rs"]
mod titulo_trocado;

// BDD-60-05: duas identidades distintas com o mesmo número SEI na árvore
// não permitem selecionar arbitrariamente um corpo. Exercita a CLI real.
#[test]
fn bdd_60_05_cli_numero_repetido_na_arvore_recusa_sem_fallback() {
    for ocultar in [false, true] {
        let mut args = vec![
            "--json",
            "documento",
            DOC_A,
            "--processo",
            NUP_A,
            "--sem-cache",
        ];
        if ocultar {
            args.push("--ocultar-inacessiveis");
        }
        let (saida, trilha) = executar_numero_repetido(&args);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "ambiguidade não pode escolher corpo: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let erro = json(&saida);
        assert_eq!(erro["subcodigo"], "numero_sei_repetido_na_arvore", "{erro}");
        assert_eq!(erro["documento"], DOC_A, "{erro}");
        assert_eq!(erro["documentos_lidos_antes"], 0, "{erro}");
        assert!(erro.get("documentos").is_none(), "{erro}");
        assert!(
            !trilha.iter().any(|p| p == "GET documento_visualizar"),
            "a recusa deve ocorrer antes de baixar qualquer corpo: {trilha:?}"
        );
        assert!(
            trilha
                .iter()
                .all(|p| p.starts_with("GET ") || p == "POST protocolo_pesquisa_rapida"),
            "sem escrita ou retry na falha: {trilha:?}"
        );
    }
}

#[test]
fn bdd_60_05_cli_recusa_url_revogada_apos_arvore_sem_tentar_alternativa() {
    // A árvore do processo ainda publica o href. Antes do GET de conteúdo,
    // o simulador muda a situação do documento no mesmo banco.
    // Não é uma transição capturada na cassette GO: é hipótese funcional.
    let (saida, trilha) = executar_revogado_ao_visualizar(&[
        "--json",
        "documento",
        DOC_A,
        "--processo",
        NUP_A,
        "--sem-cache",
    ]);

    assert_eq!(
        saida.status.code(),
        Some(1),
        "a URL revogada não deve produzir documento: stderr={} trilha={trilha:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let erro = json(&saida);
    assert_eq!(erro["status_http"], 403, "{erro}");
    assert_eq!(erro["documento"], DOC_A, "{erro}");
    assert!(erro.get("documentos").is_none(), "{erro}");
    assert_eq!(
        trilha
            .iter()
            .filter(|pedido| pedido.as_str() == "GET documento_visualizar")
            .count(),
        1,
        "uma única tentativa de conteúdo, sem retry/fallback: {trilha:?}"
    );
    assert!(
        trilha
            .iter()
            .any(|pedido| pedido == "GET procedimento_visualizar"),
        "a navegação deveria obter uma árvore antes da revogação: {trilha:?}"
    );
    assert!(
        trilha
            .iter()
            .all(|pedido| pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"),
        "recusa não autoriza POST mutador: {trilha:?}"
    );
}
