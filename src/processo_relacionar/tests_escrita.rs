//! BDD #258: remoção com um POST observado e releitura independente.
//! Oráculo: Todos test_processo_relacionar_remover_010.py @323eb01,
//! cassette PEN1; somente respostas sintéticas, zero SEI real.
use super::{Pedido, prever};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::tests_apoio::{
    ID, Pedido as PedidoHttp, Resposta, UNIDADE, abertura, ambiente, arvore, ok, r302, rodar_com,
};
use crate::propriedades_processo::tests_apoio::acao_da_barra;

const NUP_DESTINO: &str = "12345.000002/2026-01";

fn acao_da_arvore() -> String {
    acao_da_barra(&format!(
        "controlador.php?acao=procedimento_relacionar&id_procedimento={ID}&infra_hash={}",
        "a".repeat(64)
    ))
}

fn pagina(relacionado: bool) -> String {
    let hash = "a".repeat(64);
    let linha = if relacionado {
        format!(
            r##"<tr><td><a href="controlador.php?acao=procedimento_trabalhar&id_procedimento=22&infra_hash={hash}">{NUP_DESTINO}</a></td><td><a href="#ID-22" onclick="acaoRemover('22');">Remover</a></td></tr>"##
        )
    } else {
        String::new()
    };
    format!(
        r##"<html><head><script>
function acaoRemover(id){{
  if (confirm("Confirma exclusão do relacionamento?")){{
    document.getElementById('hdnInfraItemId').value=id;
    document.getElementById('frmProcedimentoListarRelacionamentos').action='controlador.php?acao=procedimento_excluir_relacionamento&acao_origem=procedimento_relacionar&arvore=1&id_procedimento={ID}&infra_hash={hash}';
    document.getElementById('frmProcedimentoListarRelacionamentos').submit();
  }}
}}
</script></head><body>
<form id="frmProcedimentoListarRelacionamentos" method="post" action="controlador.php?acao=procedimento_relacionar&id_procedimento={ID}&infra_hash={hash}">
<input type="hidden" name="hdnInfraTipoPagina" value="2" />
<input type="hidden" id="hdnInfraItemId" name="hdnInfraItemId" value="" />
<table summary="Lista de Processos Relacionados"><tr><th>Processo</th></tr>{linha}</table>
</form></body></html>"##
    )
}

fn executar(respostas: Vec<Resposta>) -> (Result<super::Previa, super::FalhaRelacionar>, Vec<PedidoHttp>) {
    let amb = ambiente();
    let destino = NumeroProcesso::novo(NUP_DESTINO).unwrap();
    rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        prever(
            sessao,
            &Pedido {
                nup,
                com: &destino,
                unidade: None,
                remover: true,
                confirmar: true,
            },
            indice,
            rede,
        )
    })
}

/// Dado um vínculo publicado e a action observada,
/// quando confirmo remover, então há um único POST modificador e
/// uma GET independente que verifica a ausência do vínculo.
#[test]
fn dado_relacionamento_quando_remover_confirmado_entao_post_unico_e_releitura() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina(true)));
    respostas.push(r302("controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H"));
    respostas.push(ok(pagina(false)));
    let (resultado, pedidos) = executar(respostas);
    assert!(resultado.is_ok(), "esperada remoção confirmada: {resultado:?}");
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos[4].metodo, "POST");
    assert!(pedidos[4].url.contains("acao=procedimento_excluir_relacionamento"));
    assert_eq!(pedidos[5].metodo, "GET");
}

/// Dado HTTP 403 no POST, quando outro agente remove o relacionamento
/// antes da releitura, então a resposta falha não comprova nossa escrita.
#[test]
fn dado_post_403_quando_remover_entao_nao_declara_sucesso_causal() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina(true)));
    respostas.push(Ok((403, String::new(), None)));
    respostas.push(ok(pagina(false))); // Seria concorrência, não prova do nosso POST.
    let (resultado, pedidos) = executar(respostas);
    assert!(resultado.is_err());
    assert_eq!(pedidos.len(), 5);
    assert_eq!(pedidos[4].metodo, "POST");
}

/// Dado HTTP 200 genérico no POST mutador, quando o SEI não confirma
/// o aceite, então nenhuma releitura transforma o POST em sucesso.
#[test]
fn dado_post_200_generico_quando_remover_entao_efeito_incerto() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina(true)));
    respostas.push(ok("<html><body>Resposta não conclusiva</body></html>".to_owned()));
    let (resultado, pedidos) = executar(respostas);
    assert!(resultado.is_err());
    assert_eq!(pedidos.len(), 5);
    assert_eq!(pedidos[4].metodo, "POST");
}

/// Dado 302 com aceite da remoção mas vínculo ainda publicado,
/// quando releio a tela, então não declaro alteração concluída.
#[test]
fn dado_302_aceito_mas_releitura_inalterada_entao_nao_confirma_remocao() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina(true)));
    respostas.push(r302("controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H"));
    respostas.push(ok(pagina(true)));
    let (resultado, pedidos) = executar(respostas);
    assert!(resultado.is_err());
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos[4].metodo, "POST");
    assert_eq!(pedidos[5].metodo, "GET");
}
