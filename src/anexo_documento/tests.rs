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
    if !cabecalhos
        .iter()
        .any(|(nome, _)| *nome == "content-disposition")
    {
        todos.push(("content-disposition", "inline; filename=\"0000101.pdf\""));
    }
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
    assert_eq!(a.nome_original, "Minuta de Contrato.pdf");
    assert_eq!(a.disposicao, Disposicao::Inline);
    assert_eq!(a.texto(), None);
    assert_eq!(a.conteudo, ConteudoDoAnexo::Nenhum);
}

#[test]
fn texto_plano_no_charset_declarado_sem_controles() {
    let a = ler(resposta(
        &[
            ("content-type", "text/plain; charset=iso-8859-1"),
            ("content-disposition", "inline; filename=\"oficio.txt\""),
        ],
        b"Of\xedcio\r\nlinha\x1b[31m\tfim",
    ))
    .unwrap();
    assert_eq!(a.mime, "text/plain");
    assert_eq!(a.texto(), Some("Ofício\nlinha\u{fffd}[31m\tfim"));
    let utf8_invalido = ler(resposta(
        &[
            ("content-type", "text/plain"),
            ("content-disposition", "inline; filename=\"oficio.txt\""),
        ],
        b"Of\xedcio",
    ));
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
        &[
            ("content-type", "text/html;;charset=iso-8859-1"),
            ("content-disposition", "inline; filename=\"despacho.html\""),
        ],
        html,
    ))
    .unwrap();
    assert_eq!(a.mime, "text/html");
    assert_eq!(a.nome_original, "despacho.html");
    assert_eq!(a.disposicao, Disposicao::Inline);
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
            ErroDisposicao::TipoDeDisposicaoNaoReconhecido,
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
        ("", ErroDisposicao::TipoDeDisposicaoNaoReconhecido),
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
    let sem_tamanho = resposta(
        &[
            ("content-type", "application/pdf"),
            ("content-disposition", "inline; filename=\"a.pdf\""),
        ],
        &[b'x'; 11],
    );
    assert_eq!(
        ler_anexo(sem_tamanho, "1", pequeno, DestinoDoAnexo::SoMetadados),
        Err(ErroAnexo::AcimaDoLimite)
    );
    let exato = resposta(
        &[
            ("content-type", "application/pdf"),
            ("content-disposition", "inline; filename=\"a.pdf\""),
        ],
        &[b'x'; 10],
    );
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
        "Ofício nº 1.pdf".to_owned()
    );
    assert_eq!(
        nome("inline; filename*=iso-8859-1'pt'Of%EDcio.pdf"),
        "Ofício.pdf".to_owned()
    );
    assert_eq!(
        nome("inline; filename=\"../../etc/passwd\""),
        "passwd".to_owned()
    );
    assert_eq!(
        nome("inline; filename=\"C:\\\\tmp\\\\x.pdf\""),
        "x.pdf".to_owned()
    );
    assert_eq!(
        nome("inline; filename=\"a\\\"b\u{7}.pdf\""),
        "a\"b.pdf".to_owned()
    );
    assert_eq!(nome("INLINE; FileName=plano.pdf"), "plano.pdf".to_owned());
    assert_eq!(
        ler(pdf(&[("content-disposition", "inline")])).unwrap_err(),
        ErroAnexo::DisposicaoForaDaFormaObservada(ErroDisposicao::NomeDeArquivoAusente)
    );
    assert_eq!(
        ler(pdf(&[("content-disposition", "inline; filename=\"..\"")])).unwrap_err(),
        ErroAnexo::DisposicaoForaDaFormaObservada(ErroDisposicao::NomeDeArquivoVazioAposSaneamento)
    );
    assert_eq!(
        ler(resposta(
            &[("content-type", "application/pdf")],
            b"%PDF-1.7 sintetico"
        ))
        .unwrap_err(),
        ErroAnexo::CabecalhoDeDisposicaoAusente
    );
    let a = ler(pdf(&[(
        "content-disposition",
        "attachment;;filename=\"x.pdf\"",
    )]))
    .unwrap();
    assert_eq!(a.disposicao, Disposicao::Attachment);
    assert_eq!(
        nome(&format!("inline; filename=\"{}.pdf\"", "a".repeat(400)))
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

#[test]
fn erro_utf8_nao_expoe_corpo_ou_nome_original_em_debug_ou_display() {
    use std::error::Error;
    use std::fmt::Write as _;

    // Mesmos bytes sintéticos no corpo e no filename* decodificado.
    let sigilo = b"SIGILO_ANEXO_719_\xff";
    let vetor_privado = format!("{sigilo:?}");
    let erros = [
        ler(resposta(
            &[
                ("content-type", "text/plain"),
                ("content-disposition", "inline; filename=\"sigilo.txt\""),
            ],
            sigilo,
        ))
        .unwrap_err(),
        ler(pdf(&[(
            "content-disposition",
            "attachment; filename*=UTF-8''SIGILO_ANEXO_719_%FF",
        )]))
        .unwrap_err(),
    ];
    assert!(matches!(
        erros[0],
        ErroAnexo::CorpoNaoUtf8NoCharsetDeclarado(_)
    ));
    assert!(matches!(
        erros[1],
        ErroAnexo::DisposicaoForaDaFormaObservada(ErroDisposicao::NomeEstendidoUtf8Invalido(_))
    ));

    for erro in &erros {
        let mut mensagem = format!("{erro:?}\n{erro}");
        let mut causa = erro.source();
        let mut achou_utf8 = false;
        while let Some(proxima) = causa {
            write!(&mut mensagem, "\n{proxima:?}\n{proxima}").unwrap();
            achou_utf8 |= proxima.is::<std::str::Utf8Error>();
            causa = proxima.source();
        }
        assert!(achou_utf8, "causa UTF-8 deve permanecer tipada");
        assert!(
            !mensagem.contains(&vetor_privado) && !mensagem.contains("SIGILO_ANEXO_719_"),
            "formatar erro e causas não pode publicar os bytes originais"
        );
    }
}

mod arquivo;
