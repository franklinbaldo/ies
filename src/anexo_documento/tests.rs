//! RED/GREEN da leitura do anexo (Refs #60, fatia 3). Respostas
//! SINTÉTICAS com a forma observada: `text/html;;charset=iso-8859-1` do
//! HTML recebido gravado em PEN1/PEN2/PEN4, `application/pdf` com
//! `inline; filename="…"` (`UnB`, pelo oráculo Todos) e `text/plain` do
//! ícone `documento_txt`. Nenhum valor real.
use super::cabecalhos::ErroDisposicao;
use super::*;
use std::io::{self, Cursor};
use std::path::PathBuf;

fn resposta(cabecalhos: &[(&str, &str)], corpo: &[u8]) -> RespostaBinaria<Cursor<Vec<u8>>> {
    RespostaBinaria {
        status: 200,
        cabecalhos: cabecalhos
            .iter()
            .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
            .collect(),
        corpo: Cursor::new(corpo.to_vec()),
    }
}

fn pdf(cabecalhos: &[(&str, &str)]) -> RespostaBinaria<Cursor<Vec<u8>>> {
    let mut todos = vec![("content-type", "application/pdf")];
    todos.extend_from_slice(cabecalhos);
    resposta(&todos, b"%PDF-1.7 sintetico")
}

fn sha(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

fn ler(r: RespostaBinaria<Cursor<Vec<u8>>>) -> Result<AnexoLido, ErroAnexo> {
    ler_anexo(
        r,
        "0000101",
        LimiteDoAnexo::default(),
        DestinoDoAnexo::SoMetadados,
    )
}

/// Leitor que falha se alguém tentar ler o corpo.
struct NaoLer;
impl Read for NaoLer {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("o corpo não pode ser lido")
    }
}

fn pasta() -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let p = std::env::temp_dir().join(format!(
        "ies-anexo-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn pdf_sem_saida_publica_metadados_e_hash_sem_texto() {
    let a = ler(pdf(&[
        (
            "content-disposition",
            "inline; filename=\"Minuta de Contrato.pdf\"",
        ),
        ("content-length", "18"),
    ]))
    .unwrap();
    assert_eq!(a.mime, "application/pdf");
    assert_eq!(a.tamanho, 18);
    assert_eq!(a.sha256, sha(b"%PDF-1.7 sintetico"));
    assert_eq!(a.nome_original.as_deref(), Some("Minuta de Contrato.pdf"));
    assert_eq!(a.disposicao, Some(Disposicao::Inline));
    assert_eq!(a.texto(), None);
    assert_eq!(a.conteudo, ConteudoDoAnexo::Nenhum);
}

#[test]
fn pdf_com_saida_vai_para_parcial_0600_e_so_conclui_sem_sobrescrever() {
    let dir = pasta();
    let mut a = ler_anexo(
        pdf(&[]),
        "0000101",
        LimiteDoAnexo::default(),
        DestinoDoAnexo::Diretorio(&dir),
    )
    .unwrap();
    let arquivo = a.tirar_arquivo().expect("parcial");
    assert_eq!(arquivo.nome(), "0000101.pdf");
    let parciais: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().collect();
    assert_eq!(parciais.len(), 1);
    let nome = parciais[0].file_name().into_string().unwrap();
    assert!(nome.starts_with(".0000101.pdf.parcial-"), "{nome}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let modo = parciais[0].metadata().unwrap().permissions().mode();
        assert_eq!(modo & 0o777, 0o600);
    }
    arquivo.concluir().unwrap();
    assert_eq!(
        std::fs::read(dir.join("0000101.pdf")).unwrap(),
        b"%PDF-1.7 sintetico"
    );
    assert_eq!(
        std::fs::read_dir(&dir).unwrap().count(),
        1,
        "parcial removido"
    );
    // De novo: o destino existe e é recusado antes do corpo.
    let r = RespostaBinaria {
        status: 200,
        cabecalhos: vec![("content-type".into(), "application/pdf".into())],
        corpo: NaoLer,
    };
    assert_eq!(
        ler_anexo(
            r,
            "0000101",
            LimiteDoAnexo::default(),
            DestinoDoAnexo::Diretorio(&dir)
        ),
        Err(ErroAnexo::ArquivoJaExiste)
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn saida_por_link_e_gravavel_por_outros_e_aceita() {
    // `--saida DIR` é da pessoa: o modo do diretório é dela.
    use std::os::unix::fs::PermissionsExt;
    let raiz = pasta();
    let real = raiz.join("real");
    std::fs::create_dir(&real).unwrap();
    let atalho = raiz.join("atalho");
    std::os::unix::fs::symlink(&real, &atalho).unwrap();
    for (bits, numero) in [(0o755, "0000103"), (0o775, "0000104"), (0o777, "0000105")] {
        std::fs::set_permissions(&real, std::fs::Permissions::from_mode(bits)).unwrap();
        let mut a = ler_anexo(
            pdf(&[]),
            numero,
            LimiteDoAnexo::default(),
            DestinoDoAnexo::Diretorio(&atalho),
        )
        .unwrap();
        a.tirar_arquivo().expect("parcial").concluir().unwrap();
        assert!(real.join(format!("{numero}.pdf")).is_file(), "{bits:o}");
    }
    assert_eq!(std::fs::read_dir(&real).unwrap().count(), 3);
    std::fs::remove_dir_all(raiz).unwrap();
}

#[test]
fn parcial_abandonado_e_removido() {
    let dir = pasta();
    let a = ler_anexo(
        pdf(&[]),
        "0000102",
        LimiteDoAnexo::default(),
        DestinoDoAnexo::Diretorio(&dir),
    )
    .unwrap();
    drop(a);
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn texto_plano_no_charset_declarado_sem_controles() {
    let a = ler(resposta(
        &[("content-type", "text/plain; charset=iso-8859-1")],
        b"Of\xedcio\r\nlinha\x1b[31m\tfim",
    ))
    .unwrap();
    assert_eq!(a.mime, "text/plain");
    assert_eq!(a.texto(), Some("Ofício\nlinha\u{fffd}[31m\tfim"));
    let utf8_invalido = ler(resposta(&[("content-type", "text/plain")], b"Of\xedcio"));
    assert!(matches!(
        utf8_invalido,
        Err(ErroAnexo::CorpoNaoUtf8NoCharsetDeclarado(_))
    ));
}

#[test]
fn html_recebido_com_parametro_vazio_rende_o_texto_visivel() {
    let html = b"<html><head><title>SEI/ORIGEM - 0403231 - Despacho</title></head>\
        <body><p>Texto do despacho recebido.</p><p>Bras\xedlia</p></body></html>";
    let a = ler(resposta(
        &[("content-type", "text/html;;charset=iso-8859-1")],
        html,
    ))
    .unwrap();
    assert_eq!(a.mime, "text/html");
    assert_eq!(a.nome_original, None);
    assert_eq!(a.disposicao, None);
    assert_eq!(a.texto(), Some("Texto do despacho recebido. Brasília"));
}

#[test]
fn html_de_login_ou_de_excecao_do_sei_nao_e_o_arquivo() {
    let login = b"<html><body><form id=\"frmLogin\"><input id=\"txtUsuario\" name=\"txtUsuario\">\
        <input type=\"password\" id=\"pwdSenha\" name=\"pwdSenha\"></form></body></html>";
    assert!(crate::tela_login::e_tela_login(&String::from_utf8_lossy(
        login
    )));
    assert_eq!(
        ler(resposta(&[("content-type", "text/html")], login)),
        Err(ErroAnexo::TelaDeLogin(crate::erro_publico::SessaoEncerrada))
    );
    let excecao =
        b"<html><body><div id=\"divInfraExcecao\">Anexo n\xe3o encontrado.</div></body></html>";
    let Err(ErroAnexo::RecusadoPeloSei(mensagem)) = ler(resposta(
        &[("content-type", "text/html; charset=iso-8859-1")],
        excecao,
    )) else {
        panic!("esperada a recusa do SEI");
    };
    assert_eq!(mensagem.as_deref(), Some("Anexo não encontrado."));
}

#[test]
fn tipo_ausente_fora_da_lista_ou_repetido_e_recusado_sem_ler_o_corpo() {
    let sem_ler = |cabecalhos: &[(&str, &str)]| {
        let r = RespostaBinaria {
            status: 200,
            cabecalhos: cabecalhos
                .iter()
                .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
                .collect(),
            corpo: NaoLer,
        };
        ler_anexo(
            r,
            "0000101",
            LimiteDoAnexo::default(),
            DestinoDoAnexo::SoMetadados,
        )
        .unwrap_err()
    };
    assert_eq!(sem_ler(&[]), ErroAnexo::SemTipoDeConteudo);
    assert_eq!(
        sem_ler(&[("content-type", " ")]),
        ErroAnexo::SemTipoDeConteudo
    );
    for fora in [
        "application/zip",
        "image/png",
        "application/pdf2",
        "text/htmlx",
    ] {
        assert_eq!(
            sem_ler(&[("content-type", fora)]),
            ErroAnexo::TipoNaoPermitido,
            "{fora}"
        );
    }
    assert_eq!(
        sem_ler(&[
            ("content-type", "application/pdf"),
            ("content-type", "application/pdf")
        ]),
        ErroAnexo::CabecalhoRepetido
    );
    assert_eq!(
        sem_ler(&[
            ("content-type", "application/pdf"),
            ("content-length", "1"),
            ("content-length", "1"),
        ]),
        ErroAnexo::CabecalhoRepetido
    );
    for (ruim, esperado) in [
        (
            "form-data; filename=\"a.pdf\"",
            ErroDisposicao::TipoDeDisposicaoNaoObservado,
        ),
        (
            "inline; filename=\"a.pdf\"; filename=\"b.pdf\"",
            ErroDisposicao::NomeDeArquivoRepetido,
        ),
        (
            "inline; filename=\"sem fim",
            ErroDisposicao::AspasSemFechamento,
        ),
        (
            "inline; filename*=KOI8-R''%E1",
            ErroDisposicao::CharsetDoNomeEstendidoNaoAceito,
        ),
        (
            "inline; filename*=UTF-8''%ZZ",
            ErroDisposicao::PercentualInvalidoNoNomeEstendido,
        ),
        ("", ErroDisposicao::TipoDeDisposicaoNaoObservado),
    ] {
        assert_eq!(
            sem_ler(&[
                ("content-type", "application/pdf"),
                ("content-disposition", ruim)
            ]),
            ErroAnexo::DisposicaoForaDaFormaObservada(esperado),
            "{ruim:?}"
        );
    }
}

#[test]
fn limite_declarado_recusa_antes_e_lido_recusa_durante() {
    let pequeno = LimiteDoAnexo::novo(10).unwrap();
    let r = RespostaBinaria {
        status: 200,
        cabecalhos: vec![
            ("content-type".into(), "application/pdf".into()),
            ("content-length".into(), "11".into()),
        ],
        corpo: NaoLer,
    };
    assert_eq!(
        ler_anexo(r, "1", pequeno, DestinoDoAnexo::SoMetadados),
        Err(ErroAnexo::AcimaDoLimite)
    );
    let sem_tamanho = resposta(&[("content-type", "application/pdf")], &[b'x'; 11]);
    assert_eq!(
        ler_anexo(sem_tamanho, "1", pequeno, DestinoDoAnexo::SoMetadados),
        Err(ErroAnexo::AcimaDoLimite)
    );
    let exato = resposta(&[("content-type", "application/pdf")], &[b'x'; 10]);
    assert_eq!(
        ler_anexo(exato, "1", pequeno, DestinoDoAnexo::SoMetadados)
            .unwrap()
            .tamanho,
        10
    );
    // Texto nunca passa de 8 MB, mesmo com limite maior.
    let grande = LimiteDoAnexo::novo(LIMITE_MAXIMO).unwrap();
    let r = RespostaBinaria {
        status: 200,
        cabecalhos: vec![
            ("content-type".into(), "text/plain".into()),
            (
                "content-length".into(),
                (crate::MAX_HTML_BYTES + 1).to_string(),
            ),
        ],
        corpo: NaoLer,
    };
    assert_eq!(
        ler_anexo(r, "1", grande, DestinoDoAnexo::SoMetadados),
        Err(ErroAnexo::AcimaDoLimite)
    );
}

#[test]
fn vazio_e_status_diferente_de_200() {
    assert_eq!(
        ler(resposta(&[("content-type", "application/pdf")], b"")),
        Err(ErroAnexo::Vazio)
    );
    let mut r = resposta(&[("content-type", "text/html")], b"<p>Erro interno</p>");
    r.status = 500;
    let Err(ErroAnexo::Transporte(FalhaTransporte::Status(s))) = ler(r) else {
        panic!("esperado o 5xx do transporte");
    };
    assert_eq!(s.status, 500);
}

#[test]
fn nome_original_saneado_e_filename_estrela_preferido() {
    let nome = |valor: &str| {
        ler(pdf(&[("content-disposition", valor)]))
            .unwrap()
            .nome_original
    };
    assert_eq!(
        nome("attachment; filename=\"a.pdf\"; filename*=UTF-8''Of%C3%ADcio%20n%C2%BA%201.pdf"),
        Some("Ofício nº 1.pdf".to_owned())
    );
    assert_eq!(
        nome("inline; filename*=iso-8859-1'pt'Of%EDcio.pdf"),
        Some("Ofício.pdf".to_owned())
    );
    assert_eq!(
        nome("inline; filename=\"../../etc/passwd\""),
        Some("passwd".to_owned())
    );
    assert_eq!(
        nome("inline; filename=\"C:\\\\tmp\\\\x.pdf\""),
        Some("x.pdf".to_owned())
    );
    assert_eq!(
        nome("inline; filename=\"a\\\"b\u{7}.pdf\""),
        Some("a\"b.pdf".to_owned())
    );
    assert_eq!(
        nome("INLINE; FileName=plano.pdf"),
        Some("plano.pdf".to_owned())
    );
    assert_eq!(nome("inline"), None);
    assert_eq!(nome("inline; filename=\"..\""), None);
    let a = ler(pdf(&[(
        "content-disposition",
        "attachment;;filename=\"x.pdf\"",
    )]))
    .unwrap();
    assert_eq!(a.disposicao, Some(Disposicao::Attachment));
    assert_eq!(
        nome(&format!("inline; filename=\"{}.pdf\"", "a".repeat(400)))
            .unwrap()
            .chars()
            .count(),
        255
    );
}

#[test]
fn limite_do_ambiente() {
    assert_eq!(
        LimiteDoAnexo::do_texto(None),
        Some(LimiteDoAnexo::default())
    );
    assert_eq!(LimiteDoAnexo::default().bytes(), 64 * 1024 * 1024);
    assert_eq!(
        LimiteDoAnexo::do_texto(Some("100")).map(LimiteDoAnexo::bytes),
        Some(100)
    );
    for ruim in ["", "0", "-1", "1e6", " 10", "2147483649", "abc"] {
        assert_eq!(LimiteDoAnexo::do_texto(Some(ruim)), None, "{ruim:?}");
    }
    assert!(LimiteDoAnexo::novo(LIMITE_MAXIMO).is_some());
    assert!(LimiteDoAnexo::novo(0).is_none());
}
