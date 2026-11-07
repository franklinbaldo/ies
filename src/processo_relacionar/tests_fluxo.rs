//! BDD #258: relacionamento já servido na tabela deve dispensar consulta AJAX.
//! Estrutura de linha e formulário observados na cassette PEN1 de relacionamentos.
//! Respostas sintéticas, sem requisições reais ao SEI.
use super::{FalhaRelacionar, Pedido, prever};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::tests_apoio::{
    ID, Pedido as PedidoHttp, Resposta, UNIDADE, abertura, ambiente, arvore, ok, rodar_com,
};
use crate::propriedades_processo::tests_apoio::acao_da_barra;
use std::fmt::Write as _;

const DESTINO: &str = "12345.000002/2026-01";

fn acao_da_arvore() -> String {
    acao_da_barra(&format!(
        "controlador.php?acao=procedimento_relacionar&id_procedimento={ID}&infra_hash={}",
        "a".repeat(64)
    ))
}

fn formulario_com_relacionados(numeros: &[(&str, &str)]) -> String {
    let hash = "a".repeat(64);
    let mut linhas = String::new();
    for (id, nup) in numeros {
        let _ = write!(
            linhas,
            "<tr><td><a href=\"controlador.php?acao=procedimento_trabalhar&id_procedimento={id}&infra_hash={hash}\">{nup}</a></td><td><a href=\"#ID-{id}\" onclick=\"acaoRemover('{id}');\">Remover</a></td></tr>"
        );
    }
    format!(
        r#"<html><script>
objAjax = new infraAjaxComplementar('txtProtocolo','controlador_ajax.php?acao_ajax=protocolo_RI1023&infra_hash={hash}');
</script><form id="frmProcedimentoListarRelacionamentos" method="post" action="controlador.php?acao=procedimento_relacionar&id_procedimento={ID}&infra_hash={hash}">
<input type="text" id="txtProtocolo" name="txtProtocolo" value=""/>
<input type="text" id="txtIdentificacaoProtocolo" name="txtIdentificacaoProtocolo" value=""/>
<input type="hidden" id="hdnIdProtocolo" name="hdnIdProtocolo" value=""/>
<table summary="Lista de Processos Relacionados">
<tr><th>Processo</th></tr>
{linhas}
</table></form></html>"#
    )
}

fn executar(respostas: Vec<Resposta>) -> (Result<super::Previa, FalhaRelacionar>, Vec<PedidoHttp>) {
    let amb = ambiente();
    let com = NumeroProcesso::novo(DESTINO).unwrap();
    rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        prever(
            sessao,
            &Pedido {
                nup,
                com: &com,
                unidade: None,
                remover: false,
                confirmar: false,
            },
            indice,
            rede,
        )
    })
}

/// Dado que a própria lista de relacionamentos já comprova o NUP pedido,
/// quando executo ies processo relacionar NUP --com DESTINO sem --confirmar,
/// então vejo "sem efeito", id do destino e quatro GET, sem POST AJAX.
#[test]
fn dado_relacionamento_ja_publicado_quando_previa_entao_sem_post_ajax() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[("22", DESTINO)])));
    let (resultado, pedidos) = executar(respostas);
    let previa = resultado.expect("a relação já publicada basta para a prévia");
    assert!(previa.sem_efeito);
    assert_eq!(previa.destino, DESTINO);
    assert_eq!(previa.id_destino.as_deref(), Some("22"));
    assert_eq!(previa.requisicoes, 4);
    assert_eq!(pedidos.len(), 4);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
}

/// Dado duas linhas diferentes com o mesmo NUP, quando peço a prévia,
/// então o IES recusa a ambiguidade sem consultar nem alterar o SEI.
#[test]
fn dado_nup_em_duas_linhas_quando_previa_entao_recusa_sem_ajax() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[
        ("22", DESTINO),
        ("23", DESTINO),
    ])));
    let (resultado, pedidos) = executar(respostas);
    assert!(matches!(
        resultado,
        Err(FalhaRelacionar::Estrutura(
            "relacionamentos_repetidos_na_tela"
        ))
    ));
    assert_eq!(pedidos.len(), 4);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
}
