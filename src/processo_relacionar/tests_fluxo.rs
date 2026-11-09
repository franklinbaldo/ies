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
    assert!(previa.sem_efeito());
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

/// Dado um pedido para relacionar o próprio NUP, quando executo a prévia,
/// então recebo a recusa sem disparar o AJAX de identificação.
#[test]
fn dado_destino_igual_origem_quando_previa_entao_recusa_sem_ajax() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[])));
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        prever(
            sessao,
            &Pedido {
                nup,
                com: nup,
                unidade: None,
                remover: false,
                confirmar: false,
            },
            indice,
            rede,
        )
    });
    assert!(matches!(
        resultado,
        Err(FalhaRelacionar::RelacionamentoConsigoMesmo)
    ));
    assert_eq!(pedidos.len(), 4);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
}

/// Dado que o AJAX identifica outro processo que não o NUP solicitado,
/// quando executo a prévia, então não aceito trocar silenciosamente o destino.
#[test]
fn dado_ajax_divergente_quando_previa_entao_recusa_destino_errado() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[])));
    respostas.push(ok(
        "<?xml version=\"1.0\"?><complementos><complemento nome=\"IdProcedimento\">22</complemento><complemento nome=\"ProtocoloProcedimentoFormatado\">99999.000099/2026-01</complemento></complementos>".to_owned(),
    ));
    let (resultado, pedidos) = executar(respostas);
    assert!(
        matches!(resultado, Err(FalhaRelacionar::DestinoDaConsultaDivergente)),
        "resultado do AJAX divergente: {resultado:?}"
    );
    assert_eq!(pedidos.len(), 5);
    assert_eq!(pedidos[4].metodo, "POST");
}

// Fonte comportamental: Todos tests/test_processo_relacionar_remover_010.py
// @323eb01, test_so_os_digitos_bastam; não reduzir protocolo alfanumérico a dígitos.
fn executar_com(
    respostas: Vec<Resposta>,
    destino: &str,
    remover: bool,
) -> (Result<super::Previa, FalhaRelacionar>, Vec<PedidoHttp>) {
    let amb = ambiente();
    let com = NumeroProcesso::novo(destino).unwrap();
    rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        prever(
            sessao,
            &Pedido {
                nup,
                com: &com,
                unidade: None,
                remover,
                confirmar: false,
            },
            indice,
            rede,
        )
    })
}

/// Dado um processo relacionado publicado com pontuação, quando peço
/// adicionar usando só letras e números, então a prévia informa sem efeito.
#[test]
fn dado_numero_sem_pontuacao_quando_ja_relacionado_entao_sem_ajax() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[("22", DESTINO)])));
    let compacto: String = DESTINO.chars().filter(|c| c.is_alphanumeric()).collect();
    let (resultado, pedidos) = executar_com(respostas, &compacto, false);
    let previa = resultado.expect("mesmo protocolo não precisa de AJAX");
    assert!(previa.sem_efeito());
    assert_eq!(previa.id_destino.as_deref(), Some("22"));
    assert_eq!(previa.destino, DESTINO);
    assert_eq!(pedidos.len(), 4);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
}

/// Dado um vínculo publicado com pontuação, quando peço remover com número
/// sem pontuação, então o mesmo vínculo é selecionado, sem POST.
#[test]
fn dado_numero_sem_pontuacao_quando_remover_entao_encontra_linha() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[("22", DESTINO)])));
    let compacto: String = DESTINO.chars().filter(|c| c.is_alphanumeric()).collect();
    let (resultado, pedidos) = executar_com(respostas, &compacto, true);
    let previa = resultado.expect("vínculo com formato alternativo deve ser encontrado");
    assert!(!previa.sem_efeito());
    assert_eq!(previa.id_destino.as_deref(), Some("22"));
    assert_eq!(previa.destino, DESTINO);
    assert_eq!(pedidos.len(), 4);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
}

/// Dado dois protocolos de mesma forma alfanumérica e nenhum texto exato,
/// quando removo, então a CLI recusa selecionar arbitrariamente um dos dois.
#[test]
fn dado_duas_pontuacoes_equivalentes_quando_remover_entao_ambiguo() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[
        ("22", "AB-C"),
        ("23", "A-BC"),
    ])));
    let (resultado, pedidos) = executar_com(respostas, "ABC", true);
    assert!(matches!(
        resultado,
        Err(FalhaRelacionar::Estrutura(
            "relacionamentos_repetidos_na_tela"
        ))
    ));
    assert_eq!(pedidos.len(), 4);
    assert!(pedidos.iter().all(|p| p.metodo == "GET"));
}

/// Dado duas formas equivalentes e uma com texto exato, quando removo
/// pela forma exata, então a linha textual vence a busca normalizada.
#[test]
fn dado_texto_exato_quando_remover_entao_prioriza_linha() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[
        ("22", "AB-C"),
        ("23", "A-BC"),
    ])));
    let (resultado, pedidos) = executar_com(respostas, "AB-C", true);
    let previa = resultado.expect("a linha exata evita ambiguidade da forma");
    assert_eq!(previa.id_destino.as_deref(), Some("22"));
    assert_eq!(pedidos.len(), 4);
}

/// Dado que o SEI serviu XML no complemento, quando executo uma prévia de
/// relacionamento ainda ausente, então o IES lê o id e o NUP do XML, com
/// um único POST de CONSULTA, sem POST modificador.
#[test]
fn dado_xml_do_complemento_quando_adicionar_entao_identifica_destino() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[])));
    respostas.push(ok(format!(
        "<?xml version=\"1.0\"?><complementos><complemento nome=\"IdProcedimento\">22</complemento><complemento nome=\"ProtocoloProcedimentoFormatado\">{DESTINO}</complemento></complementos>"
    )));
    let (resultado, pedidos) = executar(respostas);
    let previa = resultado.expect("XML do autocomplete deve ser lido como XML");
    assert!(!previa.sem_efeito());
    assert_eq!(previa.id_destino.as_deref(), Some("22"));
    assert_eq!(previa.destino, DESTINO);
    assert_eq!(previa.requisicoes, 5);
    assert_eq!(pedidos.len(), 5);
    assert_eq!(pedidos[4].metodo, "POST");
}

/// Dado o protocolo pedido sem pontuação e o SEI devolvendo a forma
/// publicada no XML, quando executo a prévia, então o id é identificado.
#[test]
fn dado_complemento_xml_formatado_quando_pedido_compacto_entao_equivale() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[])));
    respostas.push(ok(format!(
        "<?xml version=\"1.0\"?><complementos><complemento nome=\"IdProcedimento\">22</complemento><complemento nome=\"ProtocoloProcedimentoFormatado\">{DESTINO}</complemento></complementos>"
    )));
    let compacto: String = DESTINO.chars().filter(|c| c.is_alphanumeric()).collect();
    let (resultado, pedidos) = executar_com(respostas, &compacto, false);
    let previa = resultado.expect("comparação tolera pontuação, sem perder letras");
    assert_eq!(previa.destino, DESTINO);
    assert_eq!(previa.id_destino.as_deref(), Some("22"));
    assert_eq!(pedidos.len(), 5);
}

/// Dado que o SEI devolve a descrição do tipo no complemento,
/// quando executo ies processo relacionar NUP --com DESTINO sem confirmar,
/// então a prévia publica o tipo observado sem fazer POST modificador.
#[test]
fn dado_tipo_servido_no_xml_quando_previa_entao_mostra_o_tipo_do_destino() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[])));
    respostas.push(ok(format!(
        "<?xml version=\"1.0\"?><complementos><complemento nome=\"IdProcedimento\">22</complemento><complemento nome=\"ProtocoloProcedimentoFormatado\">{DESTINO}</complemento><complemento nome=\"NomeTipoProcedimento\">Processo Administrativo</complemento></complementos>"
    )));
    let (resultado, pedidos) = executar(respostas);
    let previa = resultado.expect("a descrição do tipo é observada no XML");
    assert_eq!(
        previa.tipo_destino.as_deref(),
        Some("Processo Administrativo")
    );
    assert_eq!(pedidos.len(), 5);
    assert_eq!(pedidos[4].metodo, "POST");
}

/// BDD-258-01. No PEN1 em 07/11/2026 a tela de relacionamento foi
/// servida com o formulário único, mas sem nenhuma tabela (duas prévias
/// live distintas: #258). Isso não deve bloquear a prévia de adição.
#[test]
fn dado_formulario_sem_tabela_quando_previa_entao_consulta_destino_sem_escrever() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));

    let pagina = formulario_com_relacionados(&[]);
    let (antes, depois) = pagina
        .split_once("<table summary=\"Lista de Processos Relacionados\">")
        .expect("fixture publica a tabela");
    let (_, apos) = depois.split_once("</table>").expect("tabela fechada");
    let html = format!("{antes}{apos}");
    assert!(
        !html.contains("<table"),
        "fixture deve representar a tela observada"
    );
    respostas.push(ok(html));
    respostas.push(ok(format!(
        "<?xml version=\"1.0\"?><complementos><complemento nome=\"IdProcedimento\">22</complemento><complemento nome=\"ProtocoloProcedimentoFormatado\">{DESTINO}</complemento></complementos>"
    )));
    let (resultado, pedidos) = executar(respostas);
    let previa = resultado.expect("a tela sem tabela não impede identificar destino");
    assert_eq!(previa.id_destino.as_deref(), Some("22"));
    assert!(!previa.sem_efeito());
    assert_eq!(pedidos.len(), 5);
    assert_eq!(pedidos[4].metodo, "POST"); // Consulta, não escrita.
}


/// Regressão técnica do BDD-258-01: a consulta AJAX não pode transformar
/// o id já atribuído a outro NUP na prova de vínculo do destino pedido.
/// Esta é uma hipótese de respostas inconsistentes, não observação PEN1.
#[test]
fn consulta_ajax_com_id_publicado_para_outro_nup_recusa_sem_escrita() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[acao_da_arvore()]));
    respostas.push(ok(formulario_com_relacionados(&[(
        "22",
        "99999.000099/2026-01",
    )])));
    respostas.push(ok(format!(
        "<?xml version=\"1.0\"?><complementos><complemento nome=\"IdProcedimento\">22</complemento><complemento nome=\"ProtocoloProcedimentoFormatado\">{DESTINO}</complemento></complementos>"
    )));
    let (resultado, pedidos) = executar(respostas);
    assert!(
        matches!(resultado, Err(FalhaRelacionar::DestinoDaConsultaDivergente)),
        "id 22 já pertence a outro NUP na tabela; resultado: {resultado:?}"
    );
    assert_eq!(pedidos.len(), 5);
    assert_eq!(pedidos[4].metodo, "POST"); // AJAX somente de consulta
}
