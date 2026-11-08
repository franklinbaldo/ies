//! Alterações controladas do HTTP no ensaio local da CLI real.
//! São hipóteses sintéticas; não constituem replay de cassette.

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EnsaioRemocao {
    Normal,
    IgnorarNoPost,
    TelaSemCheckbox,
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

pub(super) fn ajustar_resposta(
    resposta: Vec<u8>,
    ensaio: EnsaioRemocao,
    metodo: &str,
    acao: &str,
) -> Vec<u8> {
    if ensaio == EnsaioRemocao::TelaSemCheckbox && metodo == "GET" && acao == "procedimento_enviar"
    {
        ocultar_checkbox_na_resposta(resposta)
    } else {
        resposta
    }
}
