//! BDD #258: remoção com um POST observado e releitura independente.
//! Oráculo: Todos `test_processo_relacionar_remover_010.py` @323eb01,
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
        r#"<html><head><script>
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
</form></body></html>"#
    )
}

fn executar(
    respostas: Vec<Resposta>,
) -> (
    Result<super::Previa, super::FalhaRelacionar>,
    Vec<PedidoHttp>,
) {
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
    respostas.push(r302(
        "controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H",
    ));
    respostas.push(ok(pagina(false)));
    let (resultado, pedidos) = executar(respostas);
    assert!(
        resultado.is_ok(),
        "esperada remoção confirmada: {resultado:?}"
    );
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos[4].metodo, "POST");
    assert!(
        pedidos[4]
            .url
            .contains("acao=procedimento_excluir_relacionamento")
    );
    assert_eq!(pedidos[5].metodo, "GET");
}

/// Dado HTTP 403 no POST, quando outro agente remove o relacionamento
/// antes da releitura, então a resposta falha não comprova nossa escrita.
#[test]
fn dado_post_403_quando_remover_entao_nao_declara_sucesso_causal() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina(true)));
    respostas.push(Ok((403, String::new(), None)));
    let (resultado, pedidos) = executar(respostas);
    assert!(resultado.is_err());
    assert_eq!(pedidos.len(), 5);
    assert_eq!(pedidos[4].metodo, "POST");
}

/// Dado HTTP 200 genérico no POST mutador, quando o SEI não confirma
/// o aceite, então nenhuma releitura transforma o POST em sucesso.
#[test]
fn dado_post_200_generico_quando_remover_entao_nao_confirmado() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina(true)));
    respostas.push(ok(
        "<html><body>Resposta não conclusiva</body></html>".to_owned()
    ));
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
    respostas.push(r302(
        "controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H",
    ));
    respostas.push(ok(pagina(true)));
    let (resultado, pedidos) = executar(respostas);
    assert!(resultado.is_err());
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos[4].metodo, "POST");
    assert_eq!(pedidos[5].metodo, "GET");
}

fn pagina_adicionar(relacionado: bool) -> String {
    let hash = "a".repeat(64);
    let script = format!(
        "<script>var objAjax = new infraAjaxComplementar('txtProtocolo','controlador_ajax.php?acao_ajax=protocolo_RI1023&infra_hash={hash}');</script>"
    );
    pagina(relacionado)
        .replace("</head>", &format!("{script}</head>"))
        .replace(
            "<table summary",
            r#"<input type="text" id="txtProtocolo" name="txtProtocolo" value="" />
<input type="text" id="txtIdentificacaoProtocolo" name="txtIdentificacaoProtocolo" value="" />
<input type="hidden" id="hdnIdProtocolo" name="hdnIdProtocolo" value="" />
<button type="submit" name="sbmAdicionar" id="sbmAdicionar" value="Adicionar">Adicionar</button>
<table summary"#,
        )
}

fn executar_adicao(
    respostas: Vec<Resposta>,
) -> (
    Result<super::Previa, super::FalhaRelacionar>,
    Vec<PedidoHttp>,
) {
    let amb = ambiente();
    let destino = NumeroProcesso::novo(NUP_DESTINO).unwrap();
    rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        prever(
            sessao,
            &Pedido {
                nup,
                com: &destino,
                unidade: None,
                remover: false,
                confirmar: true,
            },
            indice,
            rede,
        )
    })
}

fn xml_destino() -> String {
    format!(
        r#"<?xml version="1.0"?><complementos><complemento nome="IdProcedimento">22</complemento><complemento nome="ProtocoloProcedimentoFormatado">{NUP_DESTINO}</complemento><complemento nome="NomeTipoProcedimento">Processo Administrativo</complemento></complementos>"#
    )
}

/// Dado destino identificado no XML e relação ausente, quando confirmo
/// adicionar, então um POST mutador é seguido de GET que comprova o vínculo.
#[test]
fn dado_destino_quando_adicionar_confirmado_entao_post_unico_e_releitura() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina_adicionar(false)));
    respostas.push(ok(xml_destino()));
    respostas.push(r302(
        "controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H",
    ));
    respostas.push(ok(pagina_adicionar(true)));
    let (resultado, pedidos) = executar_adicao(respostas);
    let previa = resultado.expect("o vínculo precisa estar confirmado por nova leitura");
    assert_eq!(previa.resultado, super::ResultadoRelacionar::Confirmado);
    assert_eq!(previa.id_destino.as_deref(), Some("22"));
    assert_eq!(previa.requisicoes, 7);
    assert_eq!(pedidos.len(), 7);
    assert_eq!(pedidos[4].metodo, "POST"); // Consulta complementar, sem mutação.
    assert_eq!(pedidos[5].metodo, "POST"); // Única escrita.
    assert!(pedidos[5].url.contains("acao=procedimento_relacionar"));
    let corpo = pedidos[5]
        .corpo
        .as_deref()
        .expect("corpo do único POST mutador");
    assert!(corpo.contains("hdnIdProtocolo=22"));
    assert!(corpo.contains("txtProtocolo="));
    assert!(corpo.contains("txtIdentificacaoProtocolo="));
    assert!(corpo.contains("sbmAdicionar=Adicionar"));
    assert_eq!(pedidos[6].metodo, "GET"); // Leitura independente.
}

/// Dado falha 403 do POST mutador, quando confirmo adicionar,
/// então o IES preserva o efeito incerto sem fazer outra escrita ou releitura.
#[test]
fn dado_post_403_quando_adicionar_entao_nao_confirmado_sem_retry() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina_adicionar(false)));
    respostas.push(ok(xml_destino()));
    respostas.push(Ok((403, String::new(), None)));
    let (resultado, pedidos) = executar_adicao(respostas);
    assert!(matches!(
        resultado,
        Err(super::FalhaRelacionar::NaoConfirmado(_))
    ));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos[5].metodo, "POST");
}

/// Dado resposta HTML genérica em vez do sinal de aceite, quando
/// adiciono, então o IES não transforma coincidência posterior em sucesso.
#[test]
fn dado_post_200_generico_quando_adicionar_entao_nao_confirma() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina_adicionar(false)));
    respostas.push(ok(xml_destino()));
    respostas.push(ok(
        "<html><body>Resposta inconclusiva</body></html>".to_owned()
    ));
    let (resultado, pedidos) = executar_adicao(respostas);
    assert!(matches!(
        resultado,
        Err(super::FalhaRelacionar::NaoConfirmado(_))
    ));
    assert_eq!(pedidos.len(), 6);
}

/// Dado o aceite da adição sem vínculo na releitura independente,
/// quando confirmo, então a saída é 3 e não há segundo POST.
#[test]
fn dado_302_aceito_mas_releitura_sem_vinculo_entao_nao_confirmado() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina_adicionar(false)));
    respostas.push(ok(xml_destino()));
    respostas.push(r302(
        "controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H",
    ));
    respostas.push(ok(pagina_adicionar(false)));
    let (resultado, pedidos) = executar_adicao(respostas);
    assert!(matches!(
        resultado,
        Err(super::FalhaRelacionar::NaoConfirmado(_))
    ));
    assert_eq!(pedidos.len(), 7);
    assert_eq!(pedidos[5].metodo, "POST");
    assert_eq!(pedidos[6].metodo, "GET");
}

/// Regressão auxiliar ao BDD-258-03: a ausência do antigo id não basta
/// quando o mesmo NUP reaparece na releitura com um id novo.
#[test]
fn releitura_remocao_nao_confirma_mesmo_nup_em_outro_id() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina(true)));
    respostas.push(r302(
        "controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H",
    ));
    let novo_id = pagina(true)
        .replace("id_procedimento=22&", "id_procedimento=23&")
        .replace("acaoRemover('22')", "acaoRemover('23')");
    respostas.push(ok(novo_id));
    let (resultado, pedidos) = executar(respostas);
    assert!(
        matches!(resultado, Err(super::FalhaRelacionar::NaoConfirmado(_))),
        "mesmo NUP ainda relacionado: {resultado:?}"
    );
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos[4].metodo, "POST");
    assert_eq!(pedidos[5].metodo, "GET");
}

fn pagina_com_outro_vinculo(com_alvo: bool) -> String {
    let hash = "a".repeat(64);
    let outra_linha = format!(
        r##"<tr><td><a href="controlador.php?acao=procedimento_trabalhar&id_procedimento=33&infra_hash={hash}">99999.000003/2026-01</a></td><td><a href="#ID-33" onclick="acaoRemover('33');">Remover</a></td></tr>"##
    );
    pagina(com_alvo).replace("</table>", &format!("{outra_linha}</table>"))
}

/// Regressão auxiliar ao BDD-258-03: uma exclusão não pode ser marcada
/// concluída se a releitura também perdeu um vínculo não solicitado.
#[test]
fn releitura_remocao_preserva_relacao_nao_solicitada() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina_com_outro_vinculo(true)));
    respostas.push(r302(
        "controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H",
    ));
    respostas.push(ok(pagina(false)));
    let (resultado, pedidos) = executar(respostas);
    assert!(
        matches!(resultado, Err(super::FalhaRelacionar::NaoConfirmado(_))),
        "outro vínculo desapareceu junto: {resultado:?}"
    );
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos[4].metodo, "POST");
    assert_eq!(pedidos[5].metodo, "GET");
}

/// Caso positivo da mesma fatia: o vínculo pedido some, o vínculo
/// vizinho segue publicado e não há mais de um POST mutador.
#[test]
fn releitura_remocao_com_outro_vinculo_preservado_confirma() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina_com_outro_vinculo(true)));
    respostas.push(r302(
        "controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H",
    ));
    respostas.push(ok(pagina_com_outro_vinculo(false)));
    let (resultado, pedidos) = executar(respostas);
    let previa = resultado.expect("remover só o vínculo pedido pode ser confirmado");
    assert_eq!(previa.resultado, super::ResultadoRelacionar::Confirmado);
    assert_eq!(previa.requisicoes, 6);
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos[4].metodo, "POST");
    assert_eq!(pedidos[5].metodo, "GET");
}

fn pagina_adicionar_com_outro_vinculo(relacionado: bool) -> String {
    let hash = "a".repeat(64);
    let outra_linha = format!(
        r#"<tr><td><a href="controlador.php?acao=procedimento_trabalhar&id_procedimento=33&infra_hash={hash}">99999.000003/2026-01</a></td></tr>"#
    );
    pagina_adicionar(relacionado).replace("</table>", &format!("{outra_linha}</table>"))
}

/// Regressão técnica do BDD-258-02: ver o destino após o POST não
/// autoriza declarar sucesso se outro relacionamento desapareceu.
#[test]
fn releitura_adicao_nao_confirma_perda_de_outro_vinculo() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina_adicionar_com_outro_vinculo(false)));
    respostas.push(ok(xml_destino()));
    respostas.push(r302(
        "controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H",
    ));
    respostas.push(ok(pagina_adicionar(true)));
    let (resultado, pedidos) = executar_adicao(respostas);
    assert!(
        matches!(resultado, Err(super::FalhaRelacionar::NaoConfirmado(_))),
        "vínculo vizinho perdido: {resultado:?}"
    );
    assert_eq!(pedidos.len(), 7);
    assert_eq!(pedidos[5].metodo, "POST");
    assert_eq!(pedidos[6].metodo, "GET");
}

/// Caso positivo: a adição pode ser confirmada quando o vínculo novo
/// aparece e o relacionamento observado anteriormente permanece.
#[test]
fn releitura_adicao_com_outro_vinculo_preservado_confirma() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(pagina_adicionar_com_outro_vinculo(false)));
    respostas.push(ok(xml_destino()));
    respostas.push(r302(
        "controlador.php?acao=procedimento_relacionar&resultado=1&infra_hash=H",
    ));
    respostas.push(ok(pagina_adicionar_com_outro_vinculo(true)));
    let (resultado, pedidos) = executar_adicao(respostas);
    let previa = resultado.expect("destino novo e vizinho preservado");
    assert_eq!(previa.resultado, super::ResultadoRelacionar::Confirmado);
    assert_eq!(pedidos.len(), 7);
    assert_eq!(pedidos[5].metodo, "POST");
    assert_eq!(pedidos[6].metodo, "GET");
}
