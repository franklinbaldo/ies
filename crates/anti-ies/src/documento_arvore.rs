//! Árvore e tela de trabalho derivadas das versões no mesmo SQLite.
//! Hipótese de integração sintética para #88; não realiza requisições ao SEI.
use crate::{Estado, Resposta, despachar};
use iesei::documento_versoes::{DocumentoSimulado, ler};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

fn html(corpo: String) -> Resposta {
    Resposta { estado: Estado::Ok, tipo_conteudo: "text/html; charset=utf-8", corpo }
}

fn js(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\'', "\\'")
        .replace('\n', "\\n").replace('\r', "\\r").replace('<', "\\u003c")
}

fn arvore(d: &DocumentoSimulado) -> String {
    let p = &d.id_procedimento;
    let id = &d.id_documento;
    let nup = js(&d.numero_processo);
    let rotulo = js(&format!("{} {}", d.tipo, d.numero));
    let numero = &d.numero;
    let hash = "e".repeat(64);
    let hash_corpo = "f".repeat(64);
    format!(r#"<html><head><script>
var Nos = new Array();
Nos[0] = new infraArvoreNo("PROCESSO","{p}",null,"controlador.php?acao=arvore_visualizar&id_procedimento={p}","ifrVisualizacao","{nup}","Processo simulado","svg/processo.svg","svg/processo.svg","svg/processo.svg",true,true,"noVisitado",null,"noVisitado","{nup}");
Nos[0].acoes = '';
Nos[0].src = '';
Nos[0].html = '';
Nos[1] = new infraArvoreNo("DOCUMENTO","{id}","{p}","about:blank","ifrVisualizacao","{rotulo}","Documento interno","svg/documento_interno.svg","svg/documento_interno.svg","svg/documento_interno.svg",true,false,null,null,"noVisitado","{numero}");
Nos[1].acoes = '<a href="controlador.php?acao=documento_versao_listar&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={p}&id_documento={id}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}" tabindex="455"><img title="Versões do Documento"/></a>';
Nos[1].src = 'controlador.php?acao=documento_visualizar&acao_origem=procedimento_visualizar&id_documento={id}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash_corpo}';
Nos[1].html = '';
</script></head><body><div id="divArvore"></div><form id="frmArvore"></form></body></html>"#)
}

fn trabalhar(d: &DocumentoSimulado) -> String {
    let p = &d.id_procedimento;
    let id = &d.id_documento;
    let unidade = d.revisoes.iter().max_by_key(|r| r.versao)
        .map_or("", |r| r.unidade.as_str())
        .replace('&', "&amp;").replace('<', "&lt;")
        .replace('>', "&gt;").replace('"', "&quot;");
    let hash = "e".repeat(64);
    format!(r##"<html><head><title>SEI - Processo</title></head><body>
<a id="lnkInfraUnidade" href="#">{unidade}</a>
<iframe id="ifrArvore" name="ifrArvore" src="controlador.php?acao=procedimento_visualizar&acao_origem=procedimento_trabalhar&acao_retorno=principal&id_procedimento={p}&id_documento={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={hash}"></iframe>
<iframe id="ifrVisualizacao" src="about:blank"></iframe></body></html>"##)
}

/// Sem snapshot, retorna 503 em vez de simular documento vazio.
/// Não fabrica links assinados reais: hashes são fixos e locais ao simulador.
#[must_use]
pub fn responder(
    metodo: &str, alvo: &str, banco: &Connection,
    instalacao: &Instalacao, cenario: &str,
) -> Resposta {
    if !super::eh_controlador_sei(alvo) { return despachar(metodo, alvo); }
    let acao = super::parametro_unico(alvo, "acao");
    if !matches!(acao, Some("procedimento_visualizar" | "procedimento_trabalhar")) {
        return despachar(metodo, alvo);
    }
    if metodo != "GET" { return Resposta::nova(Estado::MetodoNaoPermitido); }
    let id = super::parametro_unico(alvo, "id_documento")
        .or_else(|| super::parametro_unico(alvo, "id_protocolo"));
    let Some(id) = id.filter(|s| s.bytes().all(|b| b.is_ascii_digit())) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(documento)) = ler(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    if super::parametro_unico(alvo, "id_procedimento")
        .is_some_and(|id| id != documento.id_procedimento)
    { return Resposta::nova(Estado::NaoEncontrado); }
    match acao {
        Some("procedimento_visualizar") => html(arvore(&documento)),
        Some("procedimento_trabalhar") => html(trabalhar(&documento)),
        _ => unreachable!("ação validada"),
    }
}
