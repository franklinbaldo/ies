//! Casos sintéticos de transporte: prévia, POST único, releitura e incerto.
//! Casos derivados de Todos src/todos/andamento.py @323eb01.
use super::{
    ErroRegistrarAndamento, ErroTela, IncertezaDoAndamento, RespostaInesperada, registrar,
};
use crate::arvore_processo::tests_concluir::absoluta;
use crate::historico_processo::tests_apoio::{
    com_link, forma_go, link_do_historico, pagina_do_historico,
};
use crate::processo_concluir::tests_apoio::{
    Ambiente, ID, NUP, Resposta, UNIDADE, abertura, ambiente, arvore, get, ok, r302, rodar_com,
    trabalhar,
};
use crate::propriedades_processo::tests_apoio::acao_da_barra;

const HASH: &str = "9999999999999999999999999999999999999999999999999999999999999999";

fn link() -> String {
    format!(
        "controlador.php?acao=procedimento_atualizar_andamento&acao_origem=arvore_visualizar&id_procedimento={ID}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH}"
    )
}
fn tree() -> String {
    let base = arvore(&[UNIDADE], &[acao_da_barra(&link())]);
    com_link(&base, &[forma_go(&link_do_historico(ID))])
}
fn pagina(total: u64) -> String {
    pagina_do_historico(NUP, ID, 'R', total, 1)
}
fn pagina_apos() -> String {
    let original = pagina(1);
    let (_, resto) = original
        .split_once("<tr class=\"andamentoConcluido\">")
        .unwrap();
    let (linha, _) = resto.split_once("</tr>").unwrap();
    let nova = format!(
        "<tr class=\"andamentoConcluido\">{}</tr><tr class=\"andamentoConcluido\">",
        linha.replace("Andamento sintetico 0", "novo")
    );
    original
        .replacen("<tr class=\"andamentoConcluido\">", &nova, 1)
        .replace(
            "Lista de Andamentos (1 registro):",
            "Lista de Andamentos (2 registros):",
        )
}
fn form(id: &str) -> String {
    format!(
        "<html><body><form id=\"frmAtividadeListar\" method=\"post\" action=\"{action}\">\
         <input type=\"hidden\" name=\"hdnIdProtocolo\" value=\"{id}\">\
         <input type=\"hidden\" name=\"hdnIdAtividade\" value=\"\">\
         <textarea name=\"txaDescricao\"></textarea>\
         <input type=\"submit\" name=\"sbmSalvar\" value=\"Salvar\"></form></body></html>",
        action = link()
    )
}
fn respostas_previa() -> Vec<Resposta> {
    let mut respostas = abertura(tree());
    respostas.push(ok(pagina(1)));
    respostas.push(ok(form(ID)));
    respostas
}
fn rodar(
    amb: &Ambiente,
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (
    Result<super::Resultado, ErroRegistrarAndamento>,
    Vec<crate::processo_concluir::tests_apoio::Pedido>,
) {
    rodar_com(amb, respostas, |sessao, nup, indice, roteiro| {
        registrar(sessao, nup, "novo", confirmar, None, indice, roteiro)
    })
}

#[test]
fn previa_liga_barra_historico_e_formulario_sem_post() {
    let amb = ambiente();
    let (resultado, pedidos) = rodar(&amb, false, respostas_previa());
    let feito = resultado.expect("prévia da tela sintética");
    assert!(!feito.confirmado);
    assert_eq!(feito.requisicoes, 5);
    assert_eq!(feito.total_antes, 1);
    assert_eq!(feito.total_depois, None);
    assert_eq!(pedidos.len(), 5);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
    assert_eq!(pedidos.last(), Some(&get(&absoluta(&link()))));
}

#[test]
fn confirmacao_faz_exatamente_um_post_e_releitura() {
    let amb = ambiente();
    let mut respostas = respostas_previa();
    let voltar = format!(
        "controlador.php?acao=procedimento_consultar_historico&id_procedimento={ID}&infra_hash={HASH}"
    );
    respostas.push(r302(&voltar));
    respostas.push(ok(trabalhar()));
    respostas.push(ok(tree()));
    respostas.push(ok(pagina_apos()));
    let (resultado, pedidos) = rodar(&amb, true, respostas);
    let feito = resultado.expect("andamento conferido");
    assert!(feito.confirmado);
    assert_eq!(feito.total_depois, Some(2));
    assert_eq!(feito.requisicoes, 9);
    assert_eq!(pedidos.len(), 9);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
    let post = pedidos.iter().find(|p| p.metodo == "POST").unwrap();
    assert_eq!(post.url, absoluta(&link()));
    assert!(post.corpo.as_deref().unwrap().contains("txaDescricao=novo"));
    assert_eq!(pedidos.last().unwrap().metodo, "GET");
}

#[test]
fn resposta_inesperada_mas_releitura_confirma_sem_reenviar() {
    let amb = ambiente();
    let mut respostas = respostas_previa();
    respostas.push(ok("<html><body>sem redirect</body></html>".into()));
    respostas.push(ok(trabalhar()));
    respostas.push(ok(tree()));
    respostas.push(ok(pagina_apos()));
    let (resultado, pedidos) = rodar(&amb, true, respostas);
    let feito = resultado.expect("reconciliação pela leitura");
    assert_eq!(
        feito.resposta_inesperada,
        Some(RespostaInesperada::RespostaDoAndamentoNaoEsperada)
    );
    assert_eq!(feito.requisicoes, 9);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn se_historico_nao_mudou_o_efeito_e_incerto_sem_retry() {
    let amb = ambiente();
    let mut respostas = respostas_previa();
    let voltar = format!(
        "controlador.php?acao=procedimento_consultar_historico&id_procedimento={ID}&infra_hash={HASH}"
    );
    respostas.push(r302(&voltar));
    respostas.push(ok(trabalhar()));
    respostas.push(ok(tree()));
    respostas.push(ok(pagina(1)));
    let (resultado, pedidos) = rodar(&amb, true, respostas);
    assert!(matches!(
        resultado,
        Err(ErroRegistrarAndamento::NaoConfirmado {
            causa: IncertezaDoAndamento::AndamentoNaoInseridoUmaVez,
            ..
        })
    ));
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
    assert_eq!(pedidos.len(), 9);
}

#[test]
fn formulario_de_outro_processo_recusa_sem_post() {
    let amb = ambiente();
    let mut respostas = abertura(tree());
    respostas.push(ok(pagina(1)));
    respostas.push(ok(form("999")));
    let (resultado, pedidos) = rodar(&amb, true, respostas);
    assert!(matches!(
        resultado,
        Err(ErroRegistrarAndamento::FormatoDaTela(
            ErroTela::ProtocoloDiferente
        ))
    ));
    assert_eq!(pedidos.len(), 5);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
}
#[test]
fn previa_preserva_o_texto_original_do_andamento() {
    let amb = ambiente();
    let texto = "  novo\ntexto  ";
    let (resultado, pedidos) =
        rodar_com(&amb, respostas_previa(), |sessao, nup, indice, roteiro| {
            registrar(sessao, nup, texto, false, None, indice, roteiro)
        });
    let feito = resultado.expect("prévia da tela sintética");
    assert_eq!(feito.texto, texto);
    assert_eq!(pedidos.len(), 5);
    assert!(pedidos.iter().all(|pedido| pedido.metodo == "GET"));
}

#[test]
fn post_preserva_espacos_e_quebra_de_linha_sem_repetir_escrita() {
    let amb = ambiente();
    let mut respostas = respostas_previa();
    respostas.push(r302(&format!(
        "controlador.php?acao=procedimento_consultar_historico&id_procedimento={ID}&infra_hash={HASH}"
    )));
    respostas.push(ok(tree()));
    respostas.push(ok(pagina(1)));
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
        registrar(sessao, nup, "  novo\ntexto  ", true, None, indice, roteiro)
    });
    assert!(matches!(
        resultado,
        Err(ErroRegistrarAndamento::NaoConfirmado {
            causa: IncertezaDoAndamento::Releitura(_),
            ..
        })
    ));
    let postagens: Vec<_> = pedidos
        .iter()
        .filter(|pedido| pedido.metodo == "POST")
        .collect();
    assert_eq!(postagens.len(), 1);
    assert!(
        postagens[0]
            .corpo
            .as_deref()
            .unwrap()
            .contains("txaDescricao=++novo%0Atexto++")
    );
    assert_eq!(pedidos.len(), 8);
}
