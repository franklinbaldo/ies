//! Árvore e tela de trabalho derivadas das versões no mesmo SQLite.
//! Hipótese de integração sintética para #88; não realiza requisições ao SEI.
use crate::{Estado, Resposta, despachar};
use iesei::cancelamento_documento::Situacao;
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

fn arvore(d: &DocumentoSimulado, situacao: &Situacao) -> String {
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
    format!(
        r#"<html><head><script>
var Nos = new Array();
Nos[0] = new infraArvoreNo("PROCESSO","{p}",null,"controlador.php?acao=arvore_visualizar&id_procedimento={p}","ifrVisualizacao","{nup}","Processo simulado","svg/processo.svg","svg/processo.svg","svg/processo.svg",true,true,"noVisitado",null,"noVisitado","{nup}");
Nos[0].acoes = '';
Nos[0].src = '';
Nos[0].html = '';
Nos[1] = new infraArvoreNo("DOCUMENTO","{id}","{p}","about:blank","ifrVisualizacao","{rotulo}","{dica}","svg/{icone}.svg","svg/{icone}.svg","svg/{icone}.svg",true,false,null,null,"noVisitado","{numero}");
Nos[1].acoes = '<a href="controlador.php?acao=documento_versao_listar&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={p}&id_documento={id}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}" tabindex="455"><img title="Versões do Documento"/></a>{cancelar}';
Nos[1].src = 'controlador.php?acao=documento_visualizar&acao_origem=procedimento_visualizar&id_documento={id}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash_corpo}';
Nos[1].html = '';
</script></head><body><div id="divArvore"></div><form id="frmArvore"></form></body></html>"#
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
    let id = documento.or_else(|| super::parametro_unico(alvo, "id_protocolo"));
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
            html(arvore(&documento, &situacao))
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
        nos.push_str(&no_de_documento(
            indice,
            processo,
            &id,
            &documento.numero,
            &documento.tipo,
            true,
            &situacao,
        ));
        indice += 1;
        vistos.push(id);
    }
    for documento in internos {
        let id = documento.id.to_string();
        if vistos.contains(&id) {
            continue;
        }
        nos.push_str(&no_de_documento(
            indice,
            processo,
            &id,
            &documento.numero_sei,
            &documento.tipo,
            false,
            &Situacao::Disponivel,
        ));
        indice += 1;
    }
    Ok(nos)
}

fn no_de_documento(
    indice: usize,
    processo: &str,
    id: &str,
    numero: &str,
    tipo: &str,
    versoes: bool,
    situacao: &Situacao,
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
    let mut no = format!(
        r#"Nos[{indice}] = new infraArvoreNo("DOCUMENTO","{id}","{processo}","about:blank","ifrVisualizacao","{rotulo}","{dica}","svg/{icone}.svg","svg/{icone}.svg","svg/{icone}.svg",true,false,null,null,"noVisitado","{numero}");
"#
    );
    if versoes {
        let hash = "e".repeat(64);
        let hash_corpo = "f".repeat(64);
        write!(
            no,
            r#"Nos[{indice}].acoes = '<a href="controlador.php?acao=documento_versao_listar&id_procedimento={processo}&id_documento={id}&infra_hash={hash}">Versões</a>';
Nos[{indice}].src = 'controlador.php?acao=documento_visualizar&id_documento={id}&infra_hash={hash_corpo}';
"#
        )
        .expect("formatação de String");
    } else {
        write!(no, "Nos[{indice}].acoes = '';\nNos[{indice}].src = '';\n")
            .expect("formatação de String");
    }
    writeln!(no, "Nos[{indice}].html = '';").expect("formatação de String");
    no
}

#[cfg(test)]
mod testes_cancelamento_arvore_mista {
    use super::*;
    use iesei::banco_cassettes::iniciar;
    use iesei::cancelamento_documento::{Aplicacao, aplicar};
    use iesei::documento_versoes::Revisao;
    use std::num::NonZeroU32;

    /// Regressao tecnica do BDD-273-02: as duas arvores leem o mesmo
    /// cancelamento persistido, sem perder os outros documentos.
    #[test]
    fn cancelamento_altera_icone_na_arvore_mista_sem_apagar_outros_documentos() {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
        let cenario = "bdd-273";
        for (id, numero) in [("42", "0011519"), ("43", "0011520")] {
            versoes::guardar(
                &mut banco,
                cenario,
                &DocumentoSimulado {
                    instalacao: origem.clone(),
                    id_documento: id.into(),
                    id_procedimento: "10".into(),
                    numero_processo: "00001.000001/2026-01".into(),
                    numero: numero.into(),
                    tipo: "Despacho".into(),
                    revisoes: vec![Revisao {
                        versao: NonZeroU32::new(1).unwrap(),
                        usuario: "Teste".into(),
                        unidade: "ORIGEM".into(),
                        modificacao: "08/11/2026".into(),
                        html_corpo: "<p>Original</p>".into(),
                    }],
                },
            )
            .unwrap();
        }
        let antes = nos_do_processo(&banco, &origem, cenario, "10").unwrap();
        assert_eq!(antes.matches("documento_cancelado.svg").count(), 0);
        assert_eq!(
            aplicar(
                &banco,
                &origem,
                cenario,
                "42",
                "10",
                "Motivo do cancelamento"
            )
            .unwrap(),
            Aplicacao::Aplicada
        );
        let depois = nos_do_processo(&banco, &origem, cenario, "10").unwrap();
        assert_eq!(depois.matches("documento_cancelado.svg").count(), 3);
        assert_eq!(depois.matches("documento_interno.svg").count(), 3);
        assert!(depois.contains("Motivo do cancelamento"));
        assert!(depois.contains("0011519"));
        assert!(depois.contains("0011520"));
        let individual = responder(
            "GET",
            "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=42",
            &banco,
            &origem,
            cenario,
        );
        assert!(individual.corpo.contains("documento_cancelado.svg"));
    }
}
