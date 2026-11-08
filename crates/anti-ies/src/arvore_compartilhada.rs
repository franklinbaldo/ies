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
    let internos = match interno::documentos(banco, origem, cenario, id_processo) {
        Ok(documentos) => documentos,
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let externos = match externo::documentos(banco, origem, cenario, id_processo) {
        Ok(documentos) => documentos,
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let cadastrado = selecionado.is_some_and(|id| {
        internos.iter().any(|doc| doc.id.to_string() == id)
            || externos.iter().any(|doc| doc.id_publico() == id)
    });
    if original
        .as_ref()
        .is_some_and(|resposta| resposta.estado != Estado::Ok && !cadastrado)
    {
        return original.expect("resposta presente");
    }

    // A navegação de tramitação já contém versões e internos com seus links.
    // Sem snapshot de navegação, reutilizamos sua projeção de nós, sem
    // duplicar identidades e sem descartar versões anteriores ao PDF.
    // Ao selecionar o PDF, preserve também a barra e a situação da raiz
    // já publicadas pela navegação de tramitação para o mesmo processo.
    let alvo_sem_selecao = selecionado.map(|_| {
        format!("controlador.php?acao=procedimento_visualizar&id_procedimento={id_processo}")
    });
    let base = crate::processo_envio::navegacao(
        metodo,
        alvo_sem_selecao.as_deref().unwrap_or(alvo),
        banco,
        origem,
        cenario,
    );
    if base.as_ref().is_some_and(|r| r.estado != Estado::Ok) {
        return base.expect("resposta presente");
    }
    let tem_nos = base.is_some();
    let mut pagina = base.map_or_else(|| raiz(id_processo, &processo.nup), |r| r.corpo);
    let mut novos = if tem_nos {
        String::new()
    } else {
        match documento_arvore::nos_do_processo(banco, origem, cenario, id_processo) {
            Ok(nos) => nos,
            Err(estado) => return Resposta::nova(estado),
        }
    };
    let mut indice = pagina
        .lines()
        .chain(novos.lines())
        .filter(|linha| linha.starts_with("Nos[") && linha.contains(" = new infraArvoreNo("))
        .count();
    for documento in externos {
        // Espaço numérico separado: rowids internos e externos são independentes.
        let id = documento.id_publico();
        novos.push_str(&no_documento(
            indice,
            &id,
            id_processo,
            &documento.tipo,
            &documento.numero_sei,
            "documento_pdf",
        ));
        indice += 1;
    }
    let Some(posicao) = pagina.find("</script>") else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    pagina.insert_str(posicao, &novos);
    Resposta::html(pagina)
}
