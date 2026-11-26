//! Classificação do POST e comprovação por releitura independente.
use super::{ErroEditorNativo, Protocolo, VersaoDocumento};

/// Falha do transporte ou da decodificação após a tentativa de POST
/// NÃO comprova ausência de gravação. Recusas de forma anteriores ao
/// envio continuam sendo erros determinísticos.
pub(super) enum RespostaDoPost {
    Aceite(Option<VersaoDocumento>),
    Incerta,
}

pub(super) fn classificar_resposta_do_post(
    resposta: Result<Option<VersaoDocumento>, ErroEditorNativo>,
) -> Result<RespostaDoPost, ErroEditorNativo> {
    match resposta {
        Ok(versao) => Ok(RespostaDoPost::Aceite(versao)),
        Err(
            ErroEditorNativo::EnvioSemAceite(_)
            | ErroEditorNativo::Tela(_)
            | ErroEditorNativo::JsonDoEditor(_),
        ) => Ok(RespostaDoPost::Incerta),
        Err(erro) => Err(erro),
    }
}

/// Releitura do editor pode emitir vários GETs (inclusive assinatura).
/// Só a realiza quando o POST recebeu aceite da versão esperada.
/// Um erro ou aceite inconclusivo não justifica GET descartado.
pub(super) fn reabrir_apos_aceite<T>(
    aceite: Result<Option<VersaoDocumento>, ErroEditorNativo>,
    esperada: u32,
    reler: impl FnOnce() -> T,
) -> Result<Option<T>, ErroEditorNativo> {
    if aceite?.map(VersaoDocumento::numero) != Some(esperada) {
        return Ok(None);
    }
    Ok(Some(reler()))
}

pub(super) fn confere(
    anterior: &Protocolo,
    persistido: &Protocolo,
    nova: u32,
    editado: &str,
) -> bool {
    let Some(principal) = anterior.principal() else {
        return false;
    };
    persistido.versao().map(VersaoDocumento::numero) == Some(nova)
        && persistido.principal() == Some(principal)
        && anterior
            .secoes()
            .iter()
            .zip(super::htmls_para_post(
                anterior.secoes(),
                principal,
                editado,
            ))
            .all(|(secao, html)| {
                persistido
                    .secoes()
                    .iter()
                    .any(|s| s.nome == secao.nome && s.html == html)
            })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::tela_html::HtmlServido;
    use std::fmt::Write as _;

    // Hipótese sintética: versão e principal corretas não bastam se uma
    // seção vizinha desaparece ou diverge do conteúdo transportado.
    fn protocolo(versao: u32, campos: &[(&str, &str)]) -> Protocolo {
        let mut html = format!(
            "<form id='frmEditor' method='post'><input type='hidden' name='hdnVersao' value='{versao}'>\
             <input type='hidden' name='hdnIgnorarNovaVersao' value='N'>"
        );
        for (id, corpo) in campos {
            write!(html, "<textarea name='txaEditor_{id}'>{corpo}</textarea>").unwrap();
        }
        html.push_str(
            "</form><script>CKEDITOR.replace('txaEditor_12', {\"readOnly\":false});</script>",
        );
        Protocolo::ler(&HtmlServido::de_teste(&html)).unwrap()
    }

    const ANTERIOR: [(&str, &str); 5] = [
        ("4", "&lt;h1&gt;Fixo\r\r\n&lt;/h1&gt;"),
        ("5", ""),
        ("12", "&lt;p&gt;Antes&lt;/p&gt;"),
        ("6", ""),
        ("7", "&lt;hr&gt;"),
    ];
    const NOVA: [(&str, &str); 5] = [
        ("4", "&lt;h1&gt;Fixo\n&lt;/h1&gt;"),
        ("5", ""),
        ("12", "&lt;p&gt;Depois&lt;/p&gt;"),
        ("6", ""),
        ("7", "&lt;hr&gt;"),
    ];
    const EDITADO: &str = "<p>Depois</p>";

    #[test]
    fn confirma_versao_e_todas_as_secoes_transportadas() {
        let anterior = protocolo(1, &ANTERIOR);
        assert!(confere(&anterior, &protocolo(2, &NOVA), 2, EDITADO));
        let mut reordenada = NOVA;
        reordenada.reverse();
        assert!(confere(&anterior, &protocolo(2, &reordenada), 2, EDITADO));
        assert!(!confere(&anterior, &protocolo(3, &NOVA), 2, EDITADO));
    }

    #[test]
    fn nao_confirma_se_vizinha_mudou() {
        let anterior = protocolo(1, &ANTERIOR);
        let mut divergente = NOVA;
        divergente[0].1 = "&lt;h1&gt;Outro&lt;/h1&gt;";
        assert!(!confere(&anterior, &protocolo(2, &divergente), 2, EDITADO));
    }

    #[test]
    fn nao_confirma_se_vizinha_vazia_sumiu() {
        let anterior = protocolo(1, &ANTERIOR);
        let sem_vazia = [NOVA[0], NOVA[2], NOVA[3], NOVA[4]];
        assert!(!confere(&anterior, &protocolo(2, &sem_vazia), 2, EDITADO));
    }

    fn protocolo_json(versao: u32, campos: &[(&str, &str)]) -> Protocolo {
        let mut dados = serde_json::Map::new();
        for (id, corpo) in campos {
            dados.insert(
                format!("txaEditor_{id}"),
                serde_json::Value::String(html_escape::decode_html_entities(corpo).into_owned()),
            );
        }
        let config = serde_json::json!({
            "sei": {"versao": versao, "urlSalvar": "controlador_rest.php"},
            "initialData": dados,
            "rootsAttributes": {"txaEditor_12": {"principal": true, "somenteLeitura": false}}
        });
        let html = format!("<script>window.INFRA_EDITOR_CONFIG = {config};</script>");
        Protocolo::ler(&HtmlServido::de_teste(&html)).unwrap()
    }

    #[test]
    fn confere_todas_as_secoes_tambem_no_ckeditor() {
        let anterior = protocolo_json(1, &ANTERIOR);
        assert!(confere(&anterior, &protocolo_json(2, &NOVA), 2, EDITADO));
        let mut divergente = NOVA;
        divergente[0].1 = "&lt;h1&gt;Outro&lt;/h1&gt;";
        assert!(!confere(
            &anterior,
            &protocolo_json(2, &divergente),
            2,
            EDITADO
        ));
        let sem_vazia = [NOVA[0], NOVA[2], NOVA[3], NOVA[4]];
        assert!(!confere(
            &anterior,
            &protocolo_json(2, &sem_vazia),
            2,
            EDITADO
        ));
        assert!(!confere(&anterior, &protocolo_json(3, &NOVA), 2, EDITADO));
    }
}
