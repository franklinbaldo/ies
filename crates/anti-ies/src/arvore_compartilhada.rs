//! A árvore `procedimento_visualizar` apresenta o mesmo estado de documentos
//! internos e externos do processo. Hipótese sintética, sem executar JS.
use crate::{Estado, Resposta, documento_arvore, parametro_unico};
use iesei::documentos_simulados as interno;
use iesei::instalacao::Instalacao;
use iesei::pdf_externo as externo;
use rusqlite::Connection;

fn js(valor: &str) -> String {
    valor
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('<', "\\u003c")
}

fn raiz(id: &str, numero: &str) -> String {
    let numero = js(numero);
    format!(
        r#"<html><head><script>
var Nos = new Array();
Nos[0] = new infraArvoreNo("PROCESSO","{id}",null,"controlador.php?acao=arvore_visualizar&id_procedimento={id}","ifrVisualizacao","{numero}","Processo simulado","svg/processo.svg","svg/processo.svg","svg/processo.svg",true,true,"noVisitado",null,"noVisitado","{numero}");
Nos[0].acoes = '';
Nos[0].src = '';
Nos[0].html = '';
</script></head><body><div id="divArvore"></div><form id="frmArvore"></form></body></html>"#
    )
}

fn no_documento(
    indice: usize,
    id: &str,
    id_processo: &str,
    tipo: &str,
    numero: &str,
    icone: &str,
) -> String {
    let rotulo = js(&format!("{tipo} {numero}"));
    let numero = js(numero);
    format!(
        r#"Nos[{indice}] = new infraArvoreNo("DOCUMENTO","{id}","{id_processo}","about:blank","ifrVisualizacao","{rotulo}","{rotulo}","svg/{icone}.svg","svg/{icone}.svg","svg/{icone}.svg",true,false,null,null,"noVisitado","{numero}");
Nos[{indice}].acoes = '';
Nos[{indice}].src = '';
Nos[{indice}].html = '';
"#
    )
}

/// Projeta o que foi efetivamente cadastrado, não o upload temporário.
/// Quando há snapshot de versões, mantém intactos os nós e seus links.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id_processo) = parametro_unico(alvo, "id_procedimento") else {
        return documento_arvore::responder(metodo, alvo, banco, origem, cenario);
    };
    let processo = match externo::processo(banco, origem, cenario, id_processo) {
        Ok(Some(processo)) => processo,
        Ok(None) => {
            return documento_arvore::responder(metodo, alvo, banco, origem, cenario);
        }
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let selecionado =
        parametro_unico(alvo, "id_documento").or_else(|| parametro_unico(alvo, "id_protocolo"));
    let original =
        selecionado.map(|_| documento_arvore::responder(metodo, alvo, banco, origem, cenario));
    if original
        .as_ref()
        .is_some_and(|resposta| resposta.estado != Estado::Ok)
    {
        return original.expect("resposta presente");
    }
    let internos = match interno::documentos(banco, origem, cenario, id_processo) {
        Ok(documentos) => documentos,
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let externos = match externo::documentos(banco, origem, cenario, id_processo) {
        Ok(documentos) => documentos,
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let mut pagina = original.map_or_else(
        || raiz(id_processo, &processo.nup),
        |resposta| resposta.corpo,
    );
    let mut indice = pagina
        .lines()
        .filter(|linha| linha.starts_with("Nos[") && linha.contains(" = new infraArvoreNo("))
        .count();
    let mut novos = String::new();
    for documento in internos {
        let id = documento.id.to_string();
        if selecionado == Some(id.as_str()) {
            continue;
        }
        novos.push_str(&no_documento(
            indice,
            &id,
            id_processo,
            &documento.tipo,
            &documento.numero_sei,
            "documento_interno",
        ));
        indice += 1;
    }
    for documento in externos {
        // Espaço numérico separado: rowids internos e externos são independentes.
        let id = format!("8{:012}", documento.id);
        novos.push_str(&no_documento(
            indice,
            &id,
            id_processo,
            &documento.tipo,
            &documento.numero_sei,
            "documento_externo",
        ));
        indice += 1;
    }
    let Some(posicao) = pagina.find("</script>") else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    pagina.insert_str(posicao, &novos);
    Resposta::html(pagina)
}
