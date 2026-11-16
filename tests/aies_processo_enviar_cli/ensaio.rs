//! Alterações controladas do HTTP no ensaio local da CLI real.
//! São hipóteses sintéticas; não constituem replay de cassette.

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EnsaioRemocao {
    Normal,
    IgnorarNoPost,
    TelaSemCheckbox,
    TelaSemCheckboxSemFormularioValido,
    TelaSemCheckboxEnvioRecusado,
    AutocompleteIdRepetido,
}

/// Projeta uma instalação sem checkbox na tela de envio, preservando
/// o restante da resposta GET e o SQLite do anti-IES.
pub(super) fn ocultar_checkbox_na_resposta(resposta: Vec<u8>) -> Vec<u8> {
    let texto = String::from_utf8(resposta).unwrap();
    let (cabecalho, corpo) = texto.split_once("\r\n\r\n").unwrap();
    let campo = "<input type='checkbox' name='chkSinRemoverAnotacoes'>";
    assert!(corpo.contains(campo), "bancada precisa oferecer a anotação");
    let corpo = corpo.replacen(campo, "", 1);
    let cabecalho = cabecalho
        .split("\r\n")
        .map(|linha| {
            if linha.to_ascii_lowercase().starts_with("content-length:") {
                format!("Content-Length: {}", corpo.len())
            } else {
                linha.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n");
    format!("{cabecalho}\r\n\r\n{corpo}").into_bytes()
}

/// Uma única resposta do anti-IES tem o id duplicado no XML do autocomplete.
/// Mantém a resposta HTTP e o SQLite reais; modifica só a fronteira observada.
fn duplicar_id_no_autocomplete(resposta: Vec<u8>) -> Vec<u8> {
    let texto = String::from_utf8(resposta).unwrap();
    let (cabecalho, corpo) = texto.split_once("\r\n\r\n").unwrap();
    assert!(
        corpo.contains("<item id='2' "),
        "corpo do autocomplete: {corpo}"
    );
    let corpo = corpo.replacen("<item id='2' ", "<item id='2' id='99' ", 1);
    let cabecalho = cabecalho
        .split("\r\n")
        .map(|linha| {
            if linha.to_ascii_lowercase().starts_with("content-length:") {
                format!("Content-Length: {}", corpo.len())
            } else {
                linha.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n");
    format!("{cabecalho}\r\n\r\n{corpo}").into_bytes()
}

pub(super) fn ajustar_resposta(
    resposta: Vec<u8>,
    ensaio: EnsaioRemocao,
    metodo: &str,
    acao: &str,
) -> Vec<u8> {
    if ensaio == EnsaioRemocao::AutocompleteIdRepetido
        && metodo == "POST"
        && acao == "unidade_auto_completar_envio_processo"
    {
        return duplicar_id_no_autocomplete(resposta);
    }
    if matches!(
        ensaio,
        EnsaioRemocao::TelaSemCheckbox
            | EnsaioRemocao::TelaSemCheckboxSemFormularioValido
            | EnsaioRemocao::TelaSemCheckboxEnvioRecusado
    ) && metodo == "GET"
        && acao == "procedimento_enviar"
    {
        let resposta = ocultar_checkbox_na_resposta(resposta);
        if ensaio == EnsaioRemocao::TelaSemCheckboxSemFormularioValido {
            // Hipótese sintética: a tela permite escolher o destino, mas o
            // método servido não admite submissão; a limpeza é independente.
            let texto = String::from_utf8(resposta).unwrap();
            let original = "<form id='frmAtividadeListar' method='post'";
            assert!(texto.contains(original));
            texto
                .replacen(original, "<form id='frmAtividadeListar' method='noop'", 1)
                .into_bytes()
        } else {
            resposta
        }
    } else {
        resposta
    }
}
