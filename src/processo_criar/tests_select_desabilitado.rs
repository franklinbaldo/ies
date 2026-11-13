//! #245: selects desabilitados não são campos selecionáveis da criação.
use super::tests::{cadastro, lista, pedido, tela_controle};
use super::*;
use crate::processo_concluir::tests_apoio::{ambiente, ok, rodar_com};

/// Regressão derivada da estrutura de formulário no acervo Todos
/// `tests/paginas_reais/procedimento_gerar.html` @323eb01.
/// Atributo `disabled` no próprio select impede selecionar sua opção,
/// mesmo quando o HTML ainda contém uma option de nome e ID válidos.
#[test]
fn select_desabilitado_recusa_confirmacao_antes_do_post_criador() {
    let amb = ambiente();
    for (nivel, campo) in [
        (crate::dominio::NivelDeCriacao::Publico, "selAssuntos"),
        (crate::dominio::NivelDeCriacao::Restrito, "selHipoteseLegal"),
    ] {
        let mut solicitado = pedido(true);
        solicitado.nivel = nivel;
        let cadastro_desabilitado = cadastro().replace(
            &format!("name='{campo}'"),
            &format!("name='{campo}' disabled"),
        );
        let respostas = vec![
            ok(tela_controle()),
            ok(lista(false)),
            ok(cadastro_desabilitado),
        ];
        let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
            criar(
                sessao.origem,
                sessao.url_do_controle,
                sessao.cookie,
                &solicitado,
                rede,
            )
        });
        let erro = resultado.expect_err("select desabilitado não permite criação");
        match campo {
            "selAssuntos" => assert!(matches!(
                erro,
                ErroCriar::Cadastro(ErroCadastro::AssuntoDesabilitado)
            )),
            _ => assert!(matches!(
                erro,
                ErroCriar::Cadastro(ErroCadastro::HipoteseLegalDesabilitada)
            )),
        }
        assert_eq!(requests.len(), 3); // Controle, tipos, cadastro
        assert!(requests.iter().all(|req| req.metodo == "GET"));
    }
}
