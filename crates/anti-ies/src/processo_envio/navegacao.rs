//! Projeções Web de trabalho e árvore derivadas do processo persistido.
//! Nenhum token ou HTML é armazenado como estado funcional.
use super::{
    Estado, Instalacao, ProcessoSimulado, Resposta, html, ler, resposta_html, unico,
    unidade_da_sessao,
};
use rusqlite::Connection;

/// Projeta abertura de unidades na forma do nó raiz interpretada pelo IES.
/// Hipótese local revisável, com estado derivado exclusivamente do SQLite.
fn situacao_da_raiz(
    processo: &ProcessoSimulado,
    unidade_atual: &str,
    atribuicao: Option<&iesei::atribuicoes::Observacao>,
) -> String {
    let ancora = |sigla: &str| -> String {
        let sigla = html(sigla);
        format!(r#"<a alt="{sigla}" title="{sigla}" class="ancoraSigla">{sigla}</a>"#)
    };
    let com_atribuicao = |sigla: &str| -> String {
        let mut texto = ancora(sigla);
        if sigla == unidade_atual
            && let Some(observada) = atribuicao
            && !observada.login_atual.is_empty()
        {
            texto.push_str(" (atribuído para ");
            texto.push_str(&ancora(&observada.login_atual));
            texto.push(')');
        }
        texto
    };
    match processo.aberto_em.as_slice() {
        [] => "Processo não possui andamentos abertos.<br />".to_owned(),
        [unidade] => format!(
            "Processo aberto somente na unidade {}.<br />",
            com_atribuicao(unidade)
        ),
        unidades => {
            let mut texto = String::from("Processo aberto nas unidades:<br />");
            for unidade in unidades {
                texto.push_str(&com_atribuicao(unidade));
                texto.push_str("<br />");
            }
            texto.push_str("<br />");
            texto
        }
    }
}

/// O leitor do IES interpreta literais JS, não executa a árvore.
fn literal_js(texto: &str) -> String {
    texto
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn arvore_do_processo(
    processo: &ProcessoSimulado,
    unidade_atual: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<String, Estado> {
    let nos = crate::documento_arvore::nos_do_processo(banco, instalacao, cenario, &processo.id)?;
    let id = &processo.id;
    let numero = literal_js(&processo.numero);
    let atribuicao = iesei::atribuicoes::ler(banco, instalacao, cenario, id, unidade_atual)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let situacao = literal_js(&situacao_da_raiz(
        processo,
        unidade_atual,
        atribuicao.as_ref(),
    ));
    let relacionados = iesei::relacionamentos::observado(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let prazo_observado = iesei::prazos::ler(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?
        .is_some();
    let barra = literal_js(&crate::processo_barra::acoes_do_processo(
        processo,
        unidade_atual,
        relacionados,
        atribuicao.is_some(),
        prazo_observado,
    ));
    Ok(format!(
        r#"<html><head><script>
var Nos = new Array();
Nos[0] = new infraArvoreNo("PROCESSO","{id}",null,"controlador.php?acao=arvore_visualizar&id_procedimento={id}","ifrVisualizacao","{numero}","Processo simulado","svg/processo.svg","svg/processo.svg","svg/processo.svg",true,true,"noVisitado",null,"noVisitado","{numero}");
Nos[0].acoes = '{barra}';
Nos[0].src = '';
Nos[0].html = '{situacao}';
{nos}</script></head><body><div id="divArvore"></div><form id="frmArvore"></form></body></html>"#
    ))
}

fn trabalho_do_processo(processo: &ProcessoSimulado, unidade_atual: &str) -> String {
    let id = &processo.id;
    // A pesquisa por id_protocolo não pode perder a ação documental
    // já oferecida antes da tramitação. A unidade fechada não a publica.
    // A inclusão em unidade de destino exige a capacidade documental
    // correspondente, ainda não simulada fora da origem.
    let incluir = if processo.pode_incluir_documento(unidade_atual) {
        format!(
            "<a id='incluirDocumento' href='controlador.php?acao=documento_escolher_tipo&amp;id_procedimento={id}&amp;infra_hash={}'>Incluir Documento</a>",
            crate::protocolo_sintetico::INFRA_HASH
        )
    } else {
        String::new()
    };
    format!(
        "<html><head><title>SEI - Processo</title></head><body>\
         <a id='lnkInfraUnidade'>{}</a>\
         <iframe id='ifrArvore' name='ifrArvore' \
         src='controlador.php?acao=procedimento_visualizar&amp;id_procedimento={id}&amp;infra_hash={}'></iframe>\
         <iframe id='ifrVisualizacao' src='about:blank'></iframe>\
         {incluir}</body></html>",
        html(unidade_atual),
        crate::protocolo_sintetico::INFRA_HASH
    )
}

/// Consulta a árvore ou a tela de trabalho sem efeito; ausência do processo
/// delega à rota de documentos existente do mesmo cenário.
#[must_use]
pub fn navegacao(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Option<Resposta> {
    if !crate::eh_controlador_sei(alvo) {
        return None;
    }
    let acao = unico(alvo, "acao")?;
    if !matches!(acao, "procedimento_trabalhar" | "procedimento_visualizar") {
        return None;
    }
    // Mesmo sem PDF, uma seleção documental repetida não é a árvore sem seleção.
    // Valida também a tela de trabalho antes de escolher o processo do SQLite.
    if let Err(estado) = crate::documento_arvore::identidade_documental(alvo, true) {
        return Some(Resposta::nova(estado));
    }
    if unico(alvo, "id_documento").is_some() {
        return None;
    }
    // A pesquisa rápida redireciona para id_protocolo, enquanto os links
    // da árvore usam id_procedimento. Ambos identificam a mesma simulação.
    let id = match (unico(alvo, "id_procedimento"), unico(alvo, "id_protocolo")) {
        (Some(id), None) => id,
        (None, Some(id)) if acao == "procedimento_trabalhar" => id,
        _ => return None,
    };
    if !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let processo = match ler(banco, instalacao, cenario, id) {
        Ok(Some(processo)) => processo,
        Ok(None) => return None,
        Err(_) => return Some(Resposta::nova(Estado::DadosInsuficientes)),
    };
    if metodo != "GET" {
        return Some(Resposta::nova(Estado::MetodoNaoPermitido));
    }
    let unidade_atual = match unidade_da_sessao(banco, instalacao, cenario, &processo) {
        Ok(unidade) => unidade,
        Err(estado) => return Some(Resposta::nova(estado)),
    };
    if acao == "procedimento_trabalhar" {
        return Some(resposta_html(trabalho_do_processo(
            &processo,
            &unidade_atual,
        )));
    }
    Some(
        match arvore_do_processo(&processo, &unidade_atual, banco, instalacao, cenario) {
            Ok(corpo) => resposta_html(corpo),
            Err(estado) => Resposta::nova(estado),
        },
    )
}
