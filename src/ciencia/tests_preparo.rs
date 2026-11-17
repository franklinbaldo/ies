//! Testes sintéticos #271: instruções JavaScript vazias são inócuas;
//! não confundir um separador com instrução que possa interromper a chamada.
use super::{AlvoDaCiencia, ErroCiencia, atividade_do_aceite};
use crate::tela_html::HtmlServido;

#[test]
fn separadores_vazios_antes_da_recarga_do_processo() {
    let chamada = "atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42');";
    for corpo in [
        format!(";; {chamada}"),
        format!("function inicializar() {{ ;; {chamada} }}"),
    ] {
        let html = HtmlServido::de_teste(&format!("<html><script>{corpo}</script></html>"));
        assert_eq!(
            atividade_do_aceite(
                &html,
                AlvoDaCiencia::Processo {
                    id_procedimento: "10"
                }
            ),
            Ok("42".to_owned()),
            "instrução vazia não altera a alcançabilidade de atualizarArvore"
        );
    }
}

#[test]
fn separador_vazio_antes_da_recarga_do_documento() {
    let html = HtmlServido::de_teste(
        "<html><script>function inicializar() { ; atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=21&documento_visualizar_ciencias=1&id_atividade=43#ID-43'); }</script></html>",
    );
    assert_eq!(
        atividade_do_aceite(
            &html,
            AlvoDaCiencia::Documento {
                id_procedimento: "10",
                id_documento: "21"
            }
        ),
        Ok("43".to_owned())
    );
}

#[test]
fn comando_que_interrompe_antes_de_separador_continua_sem_prova() {
    let html = HtmlServido::de_teste(
        "<html><script>function inicializar() { throw new Error('parou'); ; atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento=10&procedimento_visualizar_ciencias=1&id_atividade=42#ID-42'); }</script></html>",
    );
    assert_eq!(
        atividade_do_aceite(
            &html,
            AlvoDaCiencia::Processo {
                id_procedimento: "10"
            }
        ),
        Err(ErroCiencia::AceiteSemAtividade)
    );
}
