//! Projeções Web de trabalho e árvore derivadas do processo persistido.
//! Nenhum token ou HTML é armazenado como estado funcional.
use super::{
    Estado, Instalacao, ProcessoSimulado, Resposta, html, ler, resposta_html, unico,
    unidade_da_sessao,
};
use rusqlite::Connection;
use std::fmt::Write as _;
mod trabalho;
use trabalho::trabalho_do_processo;

/// Renderização da hipótese sintética de pasta declarada, mas não carregada.
/// Os cards `UnB` mostram `PASTA`/`AGUARDE` e abertura bem-sucedida; a ausência
/// de `ABRIR_PASTAS` é cenário de falha, não transição observada na cassette.
fn no_pasta_pendente(
    pasta: &iesei::pastas_incompletas::PastaPendente,
    id: &str,
    indice_no: usize,
) -> String {
    let indice = pasta.indice;
    let indice_aguarde = indice_no + 1;
    let protocolo = &pasta.protocolo_declarado;
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    let pagina_hash = "5".repeat(32);
    format!(
        r#"var Pastas = [];
Pastas[{indice}] = [];
Pastas[{indice}]['link'] = 'controlador.php?acao=procedimento_paginar&id_procedimento={id}&no_pai=PASTA{indice}&pagina_hash={pagina_hash}&infra_hash={hash}';
Pastas[{indice}]['protocolos'] = '{protocolo}';
Nos[{indice_no}] = new infraArvoreNo("PASTA","PASTA{indice}","{id}","javascript:abrirFecharPasta('PASTA{indice}');",null,"I","Pasta {indice}",null,"svg/processo_aberto.svg","svg/processo_fechado.svg",false,true,null,null,null);
Nos[{indice_no}].carregado = false;
Nos[{indice_aguarde}] = new infraArvoreNo("AGUARDE","AGUARDE{indice}","PASTA{indice}",null,null,"Aguarde...","Aguarde...","/infra_css/svg/aguarde.svg",null,null,false,true,null,null,null);
"#
    )
}

/// Projeta abertura de unidades na forma do nó raiz interpretada pelo IES.
/// Hipótese local revisável, com estado derivado exclusivamente do SQLite.
fn situacao_da_raiz(
    processo: &ProcessoSimulado,
    unidade_atual: &str,
    atribuicao: Option<&iesei::atribuicoes::Observacao>,
    sobrestamento: Option<&iesei::sobrestamentos::Sobrestamento>,
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
    if let Some(estado) = sobrestamento {
        return format!(
            "Processo sobrestado na unidade {}.<br />",
            ancora(&estado.unidade)
        );
    }
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

/// Projeta nós de anexação sem misturar índices da pasta pendente.
fn nos_anexados(
    id: &str,
    primeiro_anexado: usize,
    anexados: &[iesei::anexacoes::Anexado],
) -> String {
    let mut nos = String::new();
    for (posicao, anexado) in anexados.iter().enumerate() {
        let numero = literal_js(&anexado.numero)
            .replace('"', r#"\\""#)
            .replace('<', r"\u003c");
        let id_anexado = &anexado.id;
        let indice = primeiro_anexado + posicao;
        let hash = crate::protocolo_sintetico::INFRA_HASH;
        write!(nos, r#"Nos[{indice}] = new infraArvoreNo("PROCESSO_ANEXADO","{id_anexado}","{id}","controlador.php?acao=arvore_visualizar&acao_origem=procedimento_visualizar&id_procedimento={id}&id_procedimento_anexado={id_anexado}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}","ifrVisualizacao","{numero}","Processo Sintético","svg/processo_anexado.svg","svg/processo_anexado.svg","svg/processo_anexado.svg",true,true,null,null,null,"{numero}");
Nos[{indice}].acoes = '';
Nos[{indice}].src = '';
Nos[{indice}].html = '';
"#).expect("formatação String");
    }
    nos
}

/// Ação condicional de ciência da árvore, sem afirmar registro inexistente.
fn acao_ciencia(id: &str, ciencia: &iesei::ciencia_processo::Estado) -> String {
    if matches!(ciencia, iesei::ciencia_processo::Estado::Registrada { .. }) {
        format!(
            "NosAcoes[31] = new infraArvoreAcao(\"CIENCIAS\",\"C\",\"{id}\",\"controlador.php?acao=protocolo_ciencia_listar&id_procedimento={id}&infra_hash={}\",\"ifrVisualizacao\",\"Visualizar Ciências\",\"svg/ciencia.svg\",true);\n",
            crate::protocolo_sintetico::INFRA_HASH
        )
    } else {
        String::new()
    }
}

/// Publica o acesso ao acompanhamento somente quando a fotografia foi observada.
fn inserir_acao_acompanhamento(
    acoes: &mut String,
    processo: &ProcessoSimulado,
    unidade: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<(), Estado> {
    if !processo.aberto_em.iter().any(|aberta| aberta == unidade) {
        return Ok(());
    }
    let id = &processo.id;
    let observado = iesei::acompanhamentos::ler(banco, instalacao, cenario, id, unidade)
        .map_err(|_| Estado::DadosInsuficientes)?;
    if observado.is_some() {
        let hash = crate::protocolo_sintetico::INFRA_HASH;
        write!(
            acoes,
            r#"<a href="controlador.php?acao=acompanhamento_gerenciar&amp;id_procedimento={id}&amp;infra_hash={hash}">Acompanhamento Especial</a>"#
        )
        .expect("formatação String");
    }
    Ok(())
}

/// Acesso a marcadores dependente da observação e da unidade aberta.
fn inserir_acao_marcadores(
    acoes: &mut String,
    processo: &ProcessoSimulado,
    unidade: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<(), Estado> {
    let id = &processo.id;
    let observado = iesei::marcadores::ler(banco, instalacao, cenario, id, unidade)
        .map_err(|_| Estado::DadosInsuficientes)?;
    if observado.is_some() && processo.aberto_em.iter().any(|aberta| aberta == unidade) {
        let hash = crate::protocolo_sintetico::INFRA_HASH;
        write!(
            acoes,
            r#"<a href="controlador.php?acao=andamento_marcador_gerenciar&amp;acao_origem=procedimento_visualizar&amp;acao_retorno=arvore_visualizar&amp;id_procedimento={id}&amp;arvore=1&amp;infra_sistema=100000100&amp;infra_unidade_atual=110000001&amp;infra_hash={hash}">Marcadores</a>"#
        ).expect("formatação String");
    }
    Ok(())
}

/// Consulta ao histórico publicada somente quando a lista foi observada.
fn link_consulta_historico(id: &str, historico_observado: bool) -> String {
    if !historico_observado {
        return String::new();
    }
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    format!(
        r##"<div id="divConsultarAndamento"><a href="#" onclick="consultarAndamento('controlador.php?acao=procedimento_consultar_historico&amp;acao_origem=arvore_visualizar&amp;acao_retorno=arvore_visualizar&amp;id_procedimento={id}&amp;arvore=1&amp;infra_sistema=100000100&amp;infra_unidade_atual=110000001&amp;infra_hash={hash}');">Consultar Andamento</a></div>"##
    )
}

fn arvore_do_processo(
    processo: &ProcessoSimulado,
    unidade_atual: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<String, Estado> {
    let (mut nos, proximo_indice) =
        crate::documento_arvore::nos_com_proximo_indice(banco, instalacao, cenario, &processo.id)?;
    let id = &processo.id;
    let pasta_pendente = iesei::pastas_incompletas::ler(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let pasta_presente = pasta_pendente.is_some();
    if let Some(pasta) = pasta_pendente {
        // Pastas e documentos coexistem no estado. O índice vem do próprio
        // gerador dos nós documentais, não da contagem inferida do HTML.
        nos.push_str(&no_pasta_pendente(&pasta, id, proximo_indice));
    }
    let anexados = iesei::anexacoes::listar(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?
        .ok_or(Estado::DadosInsuficientes)?;
    let inicio_anexados = proximo_indice + usize::from(pasta_presente) * 2;
    nos.push_str(&nos_anexados(id, inicio_anexados, &anexados));
    let numero = literal_js(&processo.numero);
    let atribuicao = iesei::atribuicoes::ler(banco, instalacao, cenario, id, unidade_atual)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let sobrestamento = iesei::sobrestamentos::consultar(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let situacao = literal_js(&situacao_da_raiz(
        processo,
        unidade_atual,
        atribuicao.as_ref(),
        sobrestamento.as_ref(),
    ));
    let relacionados = iesei::relacionamentos::observado(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let prazo_observado = iesei::prazos::ler(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?
        .is_some();
    let ciencia = iesei::ciencia_processo::consultar(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?
        .ok_or(Estado::DadosInsuficientes)?;
    let lista_ciencia = acao_ciencia(id, &ciencia);
    let historico_observado = iesei::andamentos::ler(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?
        .is_some();
    let mut acoes = crate::processo_barra::acoes_do_processo(
        processo,
        unidade_atual,
        relacionados,
        atribuicao.is_some(),
        prazo_observado,
        &ciencia,
        sobrestamento.as_ref(),
    );
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    inserir_acao_acompanhamento(
        &mut acoes,
        processo,
        unidade_atual,
        banco,
        instalacao,
        cenario,
    )?;
    inserir_acao_marcadores(
        &mut acoes,
        processo,
        unidade_atual,
        banco,
        instalacao,
        cenario,
    )?;
    let pode_atualizar = historico_observado
        && processo
            .aberto_em
            .iter()
            .any(|unidade| unidade == unidade_atual);
    if pode_atualizar {
        write!(
            acoes,
            r#"<a href="controlador.php?acao=procedimento_atualizar_andamento&amp;acao_origem=arvore_visualizar&amp;id_procedimento={id}&amp;arvore=1&amp;infra_sistema=100000100&amp;infra_unidade_atual=110000001&amp;infra_hash={hash}">Atualizar Andamento</a>"#
        )
        .expect("formatação String");
    }
    if iesei::comentarios::ler(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?
        .is_some()
    {
        write!(acoes,
            r#"<a href="controlador.php?acao=comentario_listar&amp;acao_origem=arvore_visualizar&amp;acao_retorno=arvore_visualizar&amp;id_procedimento={id}&amp;arvore=1&amp;infra_sistema=100000100&amp;infra_unidade_atual=110000001&amp;infra_hash={hash}">Comentários</a>"#
        ).expect("formatacao String");
    }
    let barra = literal_js(&acoes);
    let link_historico = link_consulta_historico(id, historico_observado);
    Ok(format!(
        r#"<html><head><script>
var Nos = new Array();
Nos[0] = new infraArvoreNo("PROCESSO","{id}",null,"controlador.php?acao=arvore_visualizar&id_procedimento={id}&infra_hash={}","ifrVisualizacao","{numero}","Processo simulado","svg/processo.svg","svg/processo.svg","svg/processo.svg",true,true,"noVisitado",null,"noVisitado","{numero}");
Nos[0].acoes = '{barra}';
Nos[0].src = '';
Nos[0].html = '{situacao}';
{lista_ciencia}{nos}</script></head><body><div id="divArvore"></div>{link_historico}<form id="frmArvore"></form></body></html>"#,
        crate::protocolo_sintetico::INFRA_HASH
    ))
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
    let Ok(validacao) = iesei::validacao_acesso::ler(banco, instalacao, cenario, id) else {
        return Some(Resposta::nova(Estado::DadosInsuficientes));
    };
    if acao == "procedimento_trabalhar" {
        return Some(resposta_html(trabalho_do_processo(
            &processo,
            &unidade_atual,
            validacao,
        )));
    }
    // Nao se pode contornar a modal pedindo diretamente a arvore.
    if matches!(
        validacao,
        Some(iesei::validacao_acesso::Validacao::Exigida | iesei::validacao_acesso::Validacao::Recusada)
    ) {
        return Some(Resposta::nova(Estado::AcessoNegado));
    }
    Some(
        match arvore_do_processo(&processo, &unidade_atual, banco, instalacao, cenario) {
            Ok(corpo) => resposta_html(corpo),
            Err(estado) => Resposta::nova(estado),
        },
    )
}
