//! Árvore e tela de trabalho derivadas das versões no mesmo SQLite.
//! Hipótese de integração sintética para #88; não realiza requisições ao SEI.
use crate::{Estado, Resposta, despachar};
use iesei::cancelamento_documento::Situacao;
use iesei::ciencia_documental::Estado as Ciencia;
use iesei::documento_versoes::{self as versoes, DocumentoSimulado, ler};
use iesei::documentos_simulados;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

fn html(corpo: String) -> Resposta {
    Resposta::html(corpo)
}

fn js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('<', "\\u003c")
}

/// A barra documental só oferece a lista quando a unidade da sessão tem
/// o processo aberto. Sem estado explícito de tramitação, preservar a
/// oferta anterior, sem inferir fechamento de um snapshot ausente.
fn lista_visivel(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_procedimento: &str,
) -> Result<bool, Estado> {
    let Some(processo) = iesei::processos::ler(banco, instalacao, cenario, id_procedimento)
        .map_err(|_| Estado::DadosInsuficientes)?
    else {
        return Ok(true);
    };
    let unidade = iesei::banco_cassettes::ler(banco, instalacao, cenario)
        .map_err(|_| Estado::DadosInsuficientes)?
        .map_or_else(
            || processo.unidade_origem.clone(),
            |controle| controle.unidade.as_str().to_owned(),
        );
    Ok(processo.aberto_em.contains(&unidade))
}

fn arvore(
    d: &DocumentoSimulado,
    situacao: &Situacao,
    ciencia: &Ciencia,
    lista_visivel: bool,
) -> String {
    let p = &d.id_procedimento;
    let id = &d.id_documento;
    let nup = js(&d.numero_processo);
    let rotulo = js(&format!("{} {}", d.tipo, d.numero));
    let numero = &d.numero;
    let hash = "e".repeat(64);
    let hash_corpo = "f".repeat(64);
    let (icone, dica, cancelar) = match situacao {
        Situacao::Disponivel => (
            "documento_interno",
            "Documento interno".to_owned(),
            format!(
                "<a href=\"controlador.php?acao=documento_cancelar&acao_origem=arvore_visualizar&id_procedimento={p}&id_documento={id}&infra_hash={}\">Cancelar Documento</a>",
                "a".repeat(64)
            ),
        ),
        Situacao::Cancelado { motivo } => (
            "documento_cancelado",
            format!("Documento Cancelado: {motivo}"),
            String::new(),
        ),
    };
    let dica = js(&dica);
    let ciencia_acao = crate::documento_ciencia::barra(ciencia);
    let lista_ciencia = crate::documento_ciencia::acao_da_lista(9, id, p, ciencia);
    let link_lista = if lista_visivel && matches!(situacao, Situacao::Disponivel) {
        format!(
            "<a href=\"controlador.php?acao=documento_versao_listar&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={p}&id_documento={id}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}\" tabindex=\"455\"><img title=\"Versões do Documento\"/></a>"
        )
    } else {
        String::new()
    };
    // "Incluir em Bloco de Assinatura" (#381), na forma do card UnB.
    let incluir_em_bloco = if matches!(situacao, Situacao::Disponivel) {
        format!(
            "<a href=\"controlador.php?acao=bloco_escolher&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={p}&id_documento={id}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}\" tabindex=\"453\"><img title=\"Incluir em Bloco de Assinatura\"/></a>"
        )
    } else {
        String::new()
    };
    let tela = format!(
        "controlador.php?acao=arvore_visualizar&id_procedimento={p}&id_documento={id}&infra_hash={}",
        crate::protocolo_sintetico::INFRA_HASH
    );
    // Um documento cancelado segue identificável, mas não publica conteúdo.
    let src = match situacao {
        Situacao::Disponivel => format!(
            "controlador.php?acao=documento_visualizar&acao_origem=procedimento_visualizar&id_documento={id}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash_corpo}"
        ),
        Situacao::Cancelado { .. } => String::new(),
    };
    format!(
        r#"<html><head><script>
var Nos = new Array();
var NosAcoes = new Array();
Nos[0] = new infraArvoreNo("PROCESSO","{p}",null,"controlador.php?acao=arvore_visualizar&id_procedimento={p}","ifrVisualizacao","{nup}","Processo simulado","svg/processo.svg","svg/processo.svg","svg/processo.svg",true,true,"noVisitado",null,"noVisitado","{nup}");
Nos[0].acoes = '';
Nos[0].src = '';
Nos[0].html = '';
Nos[1] = new infraArvoreNo("DOCUMENTO","{id}","{p}","{tela}","ifrVisualizacao","{rotulo}","{dica}","svg/{icone}.svg","svg/{icone}.svg","svg/{icone}.svg",true,false,null,null,"noVisitado","{numero}");
Nos[1].acoes = '{link_lista}{incluir_em_bloco}{cancelar}{ciencia_acao}';
Nos[1].src = '{src}';
Nos[1].html = '';
{lista_ciencia}</script></head><body><div id="divArvore"></div><form id="frmArvore"></form></body></html>"#
    )
}

fn trabalhar(d: &DocumentoSimulado) -> String {
    let p = &d.id_procedimento;
    let id = &d.id_documento;
    let unidade = d
        .revisoes
        .iter()
        .max_by_key(|r| r.versao)
        .map_or("", |r| r.unidade.as_str())
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    let hash = "e".repeat(64);
    format!(
        r##"<html><head><title>SEI - Processo</title></head><body>
<a id="lnkInfraUnidade" href="#">{unidade}</a>
<iframe id="ifrArvore" name="ifrArvore" src="controlador.php?acao=procedimento_visualizar&acao_origem=procedimento_trabalhar&acao_retorno=principal&id_procedimento={p}&id_documento={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}"></iframe>
<iframe id="ifrVisualizacao" src="about:blank"></iframe></body></html>"##
    )
}

/// Diferencia `id_documento` ausente de identidade explícita repetida ou vazia.
/// A árvore exclusivamente documental e a árvore mista aplicam a mesma regra.
pub(crate) fn identidade_documental(
    alvo: &str,
    permitir_selecao_vazia: bool,
) -> Result<Option<&str>, Estado> {
    let Some((_, consulta)) = alvo.split_once('?') else {
        return Ok(None);
    };
    // A chave sem '=' e malformada, nao ausente. Ignora-la permitiria
    // selecionar id_protocolo como fallback e devolver o DOC alheio.
    if consulta.split('&').any(|campo| campo == "id_documento") {
        return Err(Estado::RequisicaoInvalida);
    }
    let mut valores = consulta
        .split('&')
        .filter_map(|campo| campo.split_once('='))
        .filter(|(nome, _)| *nome == "id_documento")
        .map(|(_, valor)| valor);
    let primeiro = valores.next();
    if valores.next().is_some() || (primeiro == Some("") && !permitir_selecao_vazia) {
        return Err(Estado::RequisicaoInvalida);
    }
    // A árvore mista aceita id_documento= como ausência de seleção,
    // forma já consumida no BDD-332-03. Nunca aceita duas ocorrências.
    Ok(primeiro.filter(|valor| !valor.is_empty()))
}

/// Sem snapshot, retorna 503 em vez de simular documento vazio.
/// Não fabrica links assinados reais: hashes são fixos e locais ao simulador.
#[must_use]
pub fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !super::eh_controlador_sei(alvo) {
        return despachar(metodo, alvo);
    }
    let acao = super::parametro_unico(alvo, "acao");
    if !matches!(
        acao,
        Some("procedimento_visualizar" | "procedimento_trabalhar")
    ) {
        return despachar(metodo, alvo);
    }
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let documento = match identidade_documental(alvo, false) {
        Ok(documento) => documento,
        Err(estado) => return Resposta::nova(estado),
    };
    // id_protocolo e id_documento podem selecionar o mesmo documento,
    // mas valores divergentes nao autorizam escolher um deles como fallback.
    let protocolo = match super::documento_versoes::parametro(alvo, "id_protocolo") {
        Ok(protocolo) => protocolo,
        Err(estado) => return Resposta::nova(estado),
    };
    if documento.is_some() && protocolo.is_some() && documento != protocolo {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let id = documento.or(protocolo);
    let Some(id) = id.filter(|s| s.bytes().all(|b| b.is_ascii_digit())) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(documento)) = ler(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    // Ausência de processo é permitida no link observado; repetição,
    // inclusive de dois valores iguais, nunca seleciona documento.
    let processo = match super::documento_versoes::parametro(alvo, "id_procedimento") {
        Ok(processo) => processo,
        Err(estado) => return Resposta::nova(estado),
    };
    if processo.is_some_and(|id| id != documento.id_procedimento) {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    match acao {
        Some("procedimento_visualizar") => {
            let situacao = match crate::documento_cancelar::estado(
                banco,
                instalacao,
                cenario,
                &documento.id_documento,
                &documento.id_procedimento,
            ) {
                Ok(Some(situacao)) => situacao,
                Ok(None) => return Resposta::nova(Estado::DadosInsuficientes),
                Err(estado) => return Resposta::nova(estado),
            };
            let ciencia = match crate::documento_ciencia::estado(
                banco,
                instalacao,
                cenario,
                &documento.id_documento,
                &documento.id_procedimento,
            ) {
                Ok(Some(ciencia)) => ciencia,
                Ok(None) => return Resposta::nova(Estado::DadosInsuficientes),
                Err(estado) => return Resposta::nova(estado),
            };
            let lista = match lista_visivel(banco, instalacao, cenario, &documento.id_procedimento)
            {
                Ok(lista) => lista,
                Err(estado) => return Resposta::nova(estado),
            };
            html(arvore(&documento, &situacao, &ciencia, lista))
        }
        Some("procedimento_trabalhar") => html(trabalhar(&documento)),
        _ => unreachable!("ação validada"),
    }
}

/// Renderiza nós dos documentos existentes no SQLite do processo em tramitação.
/// Versões e documentos internos compartilham o mesmo pai e não desaparecem.
pub(crate) fn nos_do_processo(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    processo: &str,
) -> Result<String, Estado> {
    nos_com_proximo_indice(banco, instalacao, cenario, processo).map(|(nos, _)| nos)
}

/// Projeta a lista documental e entrega o próximo índice livre para nós de
/// pastas, sem interpretar ou reescrever o JavaScript que acabou de gerar.
pub(crate) fn nos_com_proximo_indice(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    processo: &str,
) -> Result<(String, usize), Estado> {
    let lista = lista_visivel(banco, instalacao, cenario, processo)?;
    let ids = versoes::ids_do_processo(banco, instalacao, cenario, processo)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let internos = documentos_simulados::documentos(banco, instalacao, cenario, processo)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let mut nos = String::new();
    let mut vistos = Vec::new();
    let mut indice = 1;
    for id in ids {
        let documento = versoes::ler(banco, instalacao, cenario, &id)
            .map_err(|_| Estado::DadosInsuficientes)?
            .ok_or(Estado::DadosInsuficientes)?;
        let situacao =
            crate::documento_cancelar::estado(banco, instalacao, cenario, &id, processo)?
                .ok_or(Estado::DadosInsuficientes)?;
        let ciencia = crate::documento_ciencia::estado(banco, instalacao, cenario, &id, processo)?
            .ok_or(Estado::DadosInsuficientes)?;
        nos.push_str(&no_de_documento(
            indice,
            processo,
            &id,
            &documento.numero,
            &documento.tipo,
            (true, lista),
            (&situacao, &ciencia),
        ));
        indice += 1;
        vistos.push(id);
    }
    for documento in internos {
        let id = documento.id.to_string();
        if vistos.contains(&id) {
            continue;
        }
        let situacao =
            crate::documento_cancelar::estado(banco, instalacao, cenario, &id, processo)?
                .ok_or(Estado::DadosInsuficientes)?;
        let ciencia = crate::documento_ciencia::estado(banco, instalacao, cenario, &id, processo)?
            .ok_or(Estado::DadosInsuficientes)?;
        nos.push_str(&no_de_documento(
            indice,
            processo,
            &id,
            &documento.numero_sei,
            &documento.tipo,
            (false, lista),
            (&situacao, &ciencia),
        ));
        indice += 1;
    }
    Ok((nos, indice))
}

fn no_de_documento(
    indice: usize,
    processo: &str,
    id: &str,
    numero: &str,
    tipo: &str,
    (versoes, lista_visivel): (bool, bool),
    (situacao, ciencia): (&Situacao, &Ciencia),
) -> String {
    let (icone, dica) = match situacao {
        Situacao::Disponivel => ("documento_interno", "Documento interno".to_owned()),
        Situacao::Cancelado { motivo } => (
            "documento_cancelado",
            format!("Documento Cancelado: {motivo}"),
        ),
    };
    let dica = js(&dica);
    let id = js(id);
    let processo = js(processo);
    let numero = js(numero);
    let rotulo = js(&format!("{tipo} {numero}"));
    let ciencia_acao = crate::documento_ciencia::barra(ciencia);
    let lista_ciencia =
        crate::documento_ciencia::acao_da_lista(indice + 9, &id, &processo, ciencia);
    let tela = format!(
        "controlador.php?acao=arvore_visualizar&id_procedimento={processo}&id_documento={id}&infra_hash={}",
        crate::protocolo_sintetico::INFRA_HASH
    );
    let cancelar = if matches!(situacao, Situacao::Disponivel) {
        format!(
            "<a href=\"controlador.php?acao=documento_cancelar&acao_origem=arvore_visualizar&id_procedimento={processo}&id_documento={id}&infra_hash={}\">Cancelar Documento</a>",
            crate::protocolo_sintetico::INFRA_HASH
        )
    } else {
        String::new()
    };
    let mut no = format!(
        r#"Nos[{indice}] = new infraArvoreNo("DOCUMENTO","{id}","{processo}","{tela}","ifrVisualizacao","{rotulo}","{dica}","svg/{icone}.svg","svg/{icone}.svg","svg/{icone}.svg",true,false,null,null,"noVisitado","{numero}");
"#
    );
    if versoes {
        let hash = "e".repeat(64);
        let link_lista = if lista_visivel && matches!(situacao, Situacao::Disponivel) {
            format!(
                "<a href=\"controlador.php?acao=documento_versao_listar&id_procedimento={processo}&id_documento={id}&infra_hash={hash}\">Versões</a>"
            )
        } else {
            String::new()
        };
        let hash_corpo = "f".repeat(64);
        let src = match situacao {
            Situacao::Disponivel => format!(
                "controlador.php?acao=documento_visualizar&id_documento={id}&infra_hash={hash_corpo}"
            ),
            Situacao::Cancelado { .. } => String::new(),
        };
        write!(
            no,
            "Nos[{indice}].acoes = '{link_lista}{cancelar}{ciencia_acao}';\nNos[{indice}].src = '{src}';\n"
        )
        .expect("formatação de String");
    } else {
        write!(
            no,
            "Nos[{indice}].acoes = '{cancelar}{ciencia_acao}';\nNos[{indice}].src = '';\n"
        )
        .expect("formatação de String");
    }
    writeln!(no, "Nos[{indice}].html = '';").expect("formatação de String");
    no.push_str(&lista_ciencia);
    no
}

#[cfg(test)]
#[path = "documento_arvore/testes_cancelamento_arvore_mista.rs"]
mod testes_cancelamento_arvore_mista;
