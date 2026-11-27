use super::{FalhaPrevia, Leitura, executar, executar_com_falha_previa, falha, json};

#[test]
fn exportacao_zip_preserva_documento_cadastrado_e_nao_repete_geracao() {
    use sha2::Digest as _;
    use std::io::{Cursor, Read};

    // Hipótese de saída: o ZIP integral contém o PDF previamente cadastrado.
    // O formulário e o link de download têm replay original separado.
    let (saidas, eventos, documentos, arquivo) = executar(None, false, Leitura::Zip, &[]);
    assert_eq!(documentos, 1, "exportar não cria novo documento");
    assert_eq!(saidas[0].status.code(), Some(0), "cadastro: {eventos:?}");
    assert_eq!(saidas[1].status.code(), Some(0), "releitura: {eventos:?}");
    assert_eq!(
        saidas[2].status.code(),
        Some(0),
        "exportação: stderr={} HTTP={eventos:?}",
        String::from_utf8_lossy(&saidas[2].stderr)
    );
    let bytes = arquivo
        .into_iter()
        .next()
        .flatten()
        .expect("arquivo efetivamente produzido pela CLI");
    let resultado = json(&saidas[2]);
    assert_eq!(resultado["formato"], "zip");
    assert_eq!(resultado["tamanho_bytes"], bytes.len());
    assert_eq!(
        resultado["sha256"],
        hex::encode(sha2::Sha256::digest(&bytes))
    );
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(zip.len(), 1, "um único documento foi cadastrado");
    let mut conteudo = Vec::new();
    zip.by_index(0).unwrap().read_to_end(&mut conteudo).unwrap();
    assert_eq!(conteudo, b"%PDF-1.4\n1 0 obj\n%%EOF\n");
    for evento in ["POST procedimento_gerar_zip 200", "GET exibir_arquivo 200"] {
        assert_eq!(
            eventos.iter().filter(|e| e.as_str() == evento).count(),
            1,
            "{evento}: {eventos:?}"
        );
    }
}

#[test]
fn exportacao_pdf_de_um_unico_anexo_preserva_bytes_e_nao_repete_geracao() {
    use sha2::Digest as _;

    // Hipótese mínima: um único anexo externo, sem conversão ou composição.
    // Os bytes de teste são sintéticos; não demonstram fidelidade do PDF do SEI.
    let (saidas, eventos, documentos, arquivo) = executar(None, false, Leitura::PdfProcesso, &[]);
    assert_eq!(documentos, 1, "exportar não cria novo documento");
    assert_eq!(saidas[0].status.code(), Some(0), "cadastro: {eventos:?}");
    assert_eq!(saidas[1].status.code(), Some(0), "releitura: {eventos:?}");
    assert_eq!(
        saidas[2].status.code(),
        Some(0),
        "exportação: stderr={} HTTP={eventos:?}",
        String::from_utf8_lossy(&saidas[2].stderr)
    );
    let bytes = arquivo
        .into_iter()
        .next()
        .flatten()
        .expect("arquivo efetivamente produzido pela CLI");
    assert_eq!(bytes, b"%PDF-1.4\n1 0 obj\n%%EOF\n");
    let resultado = json(&saidas[2]);
    assert_eq!(resultado["formato"], "pdf");
    assert_eq!(resultado["tamanho_bytes"], bytes.len());
    assert_eq!(
        resultado["sha256"],
        hex::encode(sha2::Sha256::digest(&bytes))
    );
    for evento in ["POST procedimento_gerar_pdf 200", "GET exibir_arquivo 200"] {
        assert_eq!(
            eventos.iter().filter(|e| e.as_str() == evento).count(),
            1,
            "{evento}: {eventos:?}"
        );
    }
}

#[test]
fn exportacao_zip_seletiva_inclui_so_o_segundo_anexo() {
    use std::io::{Cursor, Read};

    // Hipótese de saída, não replay de corpo original: anexos com o mesmo
    // nome e tipo, mas números e bytes distintos; só o segundo deve sair.
    let (saidas, eventos, documentos, arquivo) = executar(
        None,
        false,
        Leitura::ZipSelecao(&["050000002"]),
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
    );
    assert_eq!(documentos, 2);
    for saida in &saidas {
        assert!(
            saida.status.success(),
            "stderr={} HTTP={eventos:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
    }
    let arvore = json(&saidas[2]);
    let docs = arvore["documentos"].as_array().unwrap();
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0]["numero_sei"], "050000001");
    assert_eq!(docs[1]["numero_sei"], "050000002");
    assert_ne!(docs[0]["id_documento"], docs[1]["id_documento"]);
    let bytes = arquivo.into_iter().next().flatten().unwrap();
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(zip.len(), 1);
    let mut membro = zip.by_index(0).unwrap();
    assert_eq!(membro.name().unwrap().as_ref(), "050000002.pdf");
    let mut bytes = Vec::new();
    membro.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"%PDF-1.4\n2 0 obj\n%%EOF\n");
    for evento in ["POST procedimento_gerar_zip 200", "GET exibir_arquivo 200"] {
        assert_eq!(eventos.iter().filter(|e| e.as_str() == evento).count(), 1);
    }
}

#[test]
fn exportacao_zip_seletiva_documento_fora_do_processo_nao_gera_arquivo() {
    let (saidas, eventos, documentos, arquivo) = executar(
        None,
        false,
        Leitura::ZipSelecao(&["999999999"]),
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
    );
    assert_eq!(documentos, 2);
    for saida in &saidas[..3] {
        assert!(saida.status.success(), "HTTP={eventos:?}");
    }
    falha(&saidas[3], "DocumentoSelecionadoForaDaArvore");
    assert_eq!(arquivo, vec![None]);
    assert!(!eventos.iter().any(
        |e| e.starts_with("POST procedimento_gerar_zip") || e.starts_with("GET exibir_arquivo")
    ));
}

#[test]
fn exportacao_zip_seletiva_inclui_dois_de_tres_anexos() {
    use std::io::{Cursor, Read};

    let (saidas, eventos, documentos, arquivo) = executar(
        None,
        false,
        Leitura::ZipSelecao(&["050000003", "050000001"]),
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n", b"%PDF-1.4\n3 0 obj\n%%EOF\n"],
    );
    assert_eq!(documentos, 3);
    for saida in &saidas {
        assert!(
            saida.status.success(),
            "stderr={} HTTP={eventos:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
    }
    let bytes = arquivo.into_iter().next().flatten().unwrap();
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(zip.len(), 2);
    for (nome, esperado) in [
        ("050000003.pdf", b"%PDF-1.4\n3 0 obj\n%%EOF\n"),
        ("050000001.pdf", b"%PDF-1.4\n1 0 obj\n%%EOF\n"),
    ] {
        let mut membro = zip.by_name(nome).unwrap();
        let mut bytes = Vec::new();
        membro.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes.as_slice(), esperado);
    }
    for evento in ["POST procedimento_gerar_zip 200", "GET exibir_arquivo 200"] {
        assert_eq!(eventos.iter().filter(|e| e.as_str() == evento).count(), 1);
    }
}

#[test]
fn exportacao_zip_seletiva_valido_seguido_de_inexistente_nao_gera_arquivo() {
    let (saidas, eventos, documentos, arquivo) = executar(
        None,
        false,
        Leitura::ZipSelecao(&["050000001", "999999999"]),
        &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
    );
    assert_eq!(documentos, 2);
    for saida in &saidas[..3] {
        assert!(saida.status.success(), "HTTP={eventos:?}");
    }
    falha(&saidas[3], "DocumentoSelecionadoForaDaArvore");
    assert_eq!(arquivo, vec![None]);
    assert!(!eventos.iter().any(
        |e| e.starts_with("POST procedimento_gerar_zip") || e.starts_with("GET exibir_arquivo")
    ));
}

fn baixar_zip_http(
    banco: &rusqlite::Connection,
    origem: &iesei::instalacao::Instalacao,
    sufixo: &str,
) -> (String, Vec<u8>) {
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let mut cliente = TcpStream::connect(escuta.local_addr().unwrap()).unwrap();
    let (mut servidor, _) = escuta.accept().unwrap();
    let chave = ["infra", "hash"].join("_");
    write!(
        cliente,
        "GET /sei/controlador.php?acao=exibir_arquivo&id_procedimento=10&formato=zip&{chave}={}{sufixo} HTTP/1.1\r\nHost: sei.test\r\nConnection: close\r\n\r\n",
        "a".repeat(64)
    )
    .unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    anti_ies::atender_http(&mut servidor, Some((banco, origem, super::CENARIO))).unwrap();
    drop(servidor);
    let mut resposta = Vec::new();
    cliente.read_to_end(&mut resposta).unwrap();
    let fim = resposta.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let corpo = resposta.split_off(fim + 4);
    (String::from_utf8(resposta).unwrap(), corpo)
}

fn conferir_zip_http(resposta: (String, Vec<u8>), esperados: &[(&str, &[u8])]) {
    use std::io::{Cursor, Read};

    let (cabecalho, corpo) = resposta;
    assert!(cabecalho.starts_with("HTTP/1.1 200 OK\r\n"), "{cabecalho}");
    assert!(cabecalho.contains("Content-Type: application/zip\r\n"));
    assert!(cabecalho.contains("Content-Disposition: attachment;"));
    let mut zip = zip::ZipArchive::new(Cursor::new(corpo)).unwrap();
    assert_eq!(zip.len(), esperados.len());
    for &(nome, esperado) in esperados {
        let mut membro = zip.by_name(nome).unwrap();
        let mut bytes = Vec::new();
        membro.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes.as_slice(), esperado);
    }
}

#[test]
fn download_zip_http_distingue_seletor_ausente_de_malformado() {
    use iesei::pdf_externo::{cadastrar, receber_conteudo};

    // HTTP real do simulador, estado SQLite sintético; não é replay do SEI.
    let (banco, origem) = super::preparar_banco();
    let anexos: [(&str, &[u8]); 3] = [
        ("050000001.pdf", b"%PDF-1.4\n1 0 obj\n%%EOF\n"),
        ("050000002.pdf", b"%PDF-1.4\n2 0 obj\n%%EOF\n"),
        ("050000003.pdf", b"%PDF-1.4\n3 0 obj\n%%EOF\n"),
    ];
    let ids: Vec<_> = anexos
        .iter()
        .map(|&(_, bytes)| {
            let upload =
                receber_conteudo(&banco, &origem, super::CENARIO, "10", "anexo.pdf", bytes)
                    .unwrap();
            cadastrar(&banco, &origem, super::CENARIO, "10", "293", upload.id)
                .unwrap()
                .id_publico()
        })
        .collect();
    let (a, b) = (&ids[0], &ids[1]);
    for sufixo in [
        String::new(),
        format!("&documentos_extra={a}"),
        format!("&x=documentos={a}"),
    ] {
        conferir_zip_http(baixar_zip_http(&banco, &origem, &sufixo), &anexos);
    }
    conferir_zip_http(
        baixar_zip_http(&banco, &origem, &format!("&documentos={a}")),
        &anexos[..1],
    );
    conferir_zip_http(
        baixar_zip_http(&banco, &origem, &format!("&documentos={a},{b}")),
        &anexos[..2],
    );
    for sufixo in [
        "&documentos".to_owned(),
        "&documentos=".to_owned(),
        format!("&documentos={a}&documentos={b}"),
        format!("&documentos&documentos={a}"),
        format!("&documentos={a}&documentos"),
        format!("&documentos={a}&documentos="),
        format!("&documentos={a},{a}"),
        "&documentos=9999999999999".to_owned(),
        format!("&documentos={a},"),
    ] {
        let (cabecalho, corpo) = baixar_zip_http(&banco, &origem, &sufixo);
        assert!(
            cabecalho.starts_with("HTTP/1.1 400 Bad Request\r\n"),
            "{sufixo}: {cabecalho}"
        );
        assert!(!cabecalho.contains("Content-Disposition:"));
        assert_eq!(corpo, br#"{"erro":"requisicao_invalida"}"#);
    }
}

// Intervenção sintética na projeção HTTP, não no cadastro SQLite:
// o segundo nó passa a publicar a forma fechada observada no acervo.
// O gerador real do anti-IES continua disponível para o primeiro anexo.
pub(super) fn fechar_segundo_no(resposta: &mut [u8]) -> bool {
    let prefixo = b"Nos[2].src = '";
    let Some(inicio) = resposta.windows(prefixo.len()).position(|w| w == prefixo) else {
        return false;
    };
    let resto = &mut resposta[inicio..];
    let fim = resto.iter().position(|&b| b == b'\n').unwrap();
    let linha = &mut resto[..fim];
    assert!(linha.starts_with(b"Nos[2].src = 'controlador.php?acao=documento_download_anexo"));
    let fechado = b"Nos[2].src = '';\n";
    // Preserva o Content-Length; o padding fica em outra linha,
    // pois o leitor exige o ponto e vírgula no fim da atribuição.
    linha.fill(b' ');
    linha[..fechado.len()].copy_from_slice(fechado);
    true
}

// Mesmo estado sintético e intervenção nas três execuções; apenas a
// seleção muda. A positiva precisa passar pelo gerador e baixar o ZIP.
// O replay original UnB/UFRJ da forma fechada é um teste separado.
#[test]
fn exportacao_seletiva_recusa_no_fechado_antes_do_gerador() {
    use std::io::{Cursor, Read};

    for (pedidos, valido) in [
        (&["050000001"][..], true),
        (&["050000002"][..], false),
        (&["050000001", "050000002"][..], false),
    ] {
        let (saidas, eventos, documentos, arquivos) = executar_com_falha_previa(
            None,
            Some(FalhaPrevia::DocumentoFechado),
            false,
            Leitura::ZipSelecao(pedidos),
            &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
        );
        assert_eq!(documentos, 2);
        for saida in &saidas[..3] {
            assert!(saida.status.success(), "HTTP={eventos:?}");
        }
        let arvore = json(&saidas[2]);
        let docs = arvore["documentos"].as_array().unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0]["numero_sei"], "050000001");
        assert_eq!(docs[1]["numero_sei"], "050000002");
        assert!(eventos.iter().any(|e| e == "arvore mista projetada"));
        if valido {
            assert!(saidas[3].status.success(), "HTTP={eventos:?}");
            let bytes = arquivos.into_iter().next().flatten().unwrap();
            let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
            assert_eq!(zip.len(), 1);
            let mut membro = zip.by_name("050000001.pdf").unwrap();
            let mut conteudo = Vec::new();
            membro.read_to_end(&mut conteudo).unwrap();
            assert_eq!(conteudo, b"%PDF-1.4\n1 0 obj\n%%EOF\n");
        } else {
            falha(&saidas[3], "DocumentoSelecionadoInacessivel");
            assert_eq!(arquivos, vec![None]);
        }
        for prefixo in [
            "GET procedimento_gerar_zip",
            "POST procedimento_gerar_zip",
            "GET exibir_arquivo",
        ] {
            assert_eq!(
                eventos.iter().filter(|e| e.starts_with(prefixo)).count(),
                usize::from(valido),
                "{prefixo}: {eventos:?}"
            );
        }
    }
}

// Mesmo pedido e estado, com e sem mutação do download: o ZIP errado
// tem cardinalidade correta, formato legível e o mesmo tamanho do bom.
#[test]
fn exportacao_seletiva_recusa_membro_alheio_sem_publicar() {
    for mutante in [false, true] {
        let (saidas, eventos, documentos, arquivos) = executar_com_falha_previa(
            None,
            mutante.then_some(FalhaPrevia::ZipMembroAlheio),
            false,
            Leitura::ZipSelecao(&["050000001", "050000002"]),
            &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
        );
        assert_eq!(documentos, 2);
        for saida in &saidas[..3] {
            assert!(saida.status.success(), "HTTP={eventos:?}");
        }
        for evento in ["POST procedimento_gerar_zip 200", "GET exibir_arquivo 200"] {
            assert_eq!(eventos.iter().filter(|e| e.as_str() == evento).count(), 1);
        }
        assert_eq!(
            eventos.iter().any(|e| e == "zip membro alheio projetado"),
            mutante
        );
        if mutante {
            falha(&saidas[3], "SelecaoZipDivergente");
            assert_eq!(arquivos, vec![None]);
        } else {
            assert!(saidas[3].status.success(), "HTTP={eventos:?}");
            assert!(arquivos[0].is_some());
        }
    }
}
