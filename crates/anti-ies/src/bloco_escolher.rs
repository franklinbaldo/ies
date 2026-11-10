//! Tela `bloco_escolher` ("Incluir em Bloco de Assinatura", #381) gerada do
//! SQLite: `frmBlocoEscolher` com `selBloco` (blocos de assinatura da
//! unidade) e as caixas dos documentos do processo, na forma do card
//! `bloco-incluir-anotar-retirar-unb`. POST de inclusão causal sintética;
//! nomes e seleção são derivados do conjunto de documentos no SQLite.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::bloco_detalhe::{self, Item};
use iesei::blocos::{self, Tipo};
use iesei::documento_versoes;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

const HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" && metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let (Some(id_procedimento), Some(id_documento)) = (
        parametro_unico(alvo, "id_procedimento"),
        parametro_unico(alvo, "id_documento"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let documento = match documento_versoes::ler(banco, instalacao, cenario, id_documento) {
        Ok(Some(d)) if d.id_procedimento == id_procedimento => d,
        _ => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let Ok(pagina) = blocos::ler(banco, instalacao, cenario, Tipo::Assinatura) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let linhas = pagina.map(|p| p.linhas).unwrap_or_default();
    // Documentos do processo publicados no mesmo estado derivado.
    let Ok(ids) = documento_versoes::ids_do_processo(banco, instalacao, cenario, id_procedimento)
    else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let mut documentos = Vec::new();
    for id in ids {
        match documento_versoes::ler(banco, instalacao, cenario, &id) {
            Ok(Some(d)) if d.id_procedimento == id_procedimento => documentos.push(d),
            _ => return Resposta::nova(Estado::DadosInsuficientes),
        }
    }
    if documentos.is_empty() {
        return Resposta::nova(Estado::DadosInsuficientes);
    }
    if metodo == "POST" {
        let campo = |nome| crate::documentos::campo_formulario_unico(corpo, nome);
        let (Some(numero), Some(selecionados), Some(botao)) = (
            campo("selBloco"),
            campo("hdnDocumentosItensSelecionados"),
            campo("sbmIncluir"),
        ) else {
            return Resposta::nova(Estado::RequisicaoInvalida);
        };
        let marcados: Vec<_> = documentos
            .iter()
            .enumerate()
            .filter_map(|(i, d)| {
                let nome = format!("chkDocumentosItem{i}");
                campo(&nome).map(|id| (d, id.to_owned()))
            })
            .collect();
        let todos = documentos
            .iter()
            .map(|d| d.id_documento.as_str())
            .collect::<Vec<_>>()
            .join(",");
        if !linhas.iter().any(|b| b.numero == numero)
            || marcados.len() != 1
            || marcados[0].0.id_documento != documento.id_documento
            || marcados[0].1 != documento.id_documento
            || selecionados != documento.id_documento
            || botao != "Incluir"
            || campo("hdnDocumentosItens").as_deref() != Some(todos.as_str())
        {
            return Resposta::nova(Estado::RequisicaoInvalida);
        }
        let item = Item {
            processo: documento.numero_processo,
            documento: Some(documento.numero),
            tipo: Some(documento.tipo),
            anotacoes: String::new(),
        };
        return match bloco_detalhe::incluir_documento(banco, instalacao, cenario, &numero, &item) {
            Ok(true) => Resposta::redirecionar(format!(
                "controlador.php?acao=bloco_escolher&acao_origem=bloco_escolher&arvore=1&id_procedimento={id_procedimento}&id_documento={id_documento}&infra_hash={HASH}"
            )),
            Ok(false) => Resposta::nova(Estado::RequisicaoInvalida),
            Err(_) => Resposta::nova(Estado::DadosInsuficientes),
        };
    }
    let mut opcoes = String::from("<option value=\"null\" selected=\"selected\">&nbsp;</option>\n");
    for linha in &linhas {
        writeln!(
            opcoes,
            "<option value=\"{n}\">{n} - {d}</option>",
            n = escapar(&linha.numero),
            d = escapar(&linha.descricao)
        )
        .expect("escrever em String");
    }
    let id = escapar(&documento.id_documento);
    let p = escapar(id_procedimento);
    let ids = documentos
        .iter()
        .map(|d| d.id_documento.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let mut caixas = String::new();
    for (i, d) in documentos.iter().enumerate() {
        writeln!(
            caixas,
            r#"<tr><td><input {marcado} class="infraCheckbox" id="chkDocumentosItem{i}" name="chkDocumentosItem{i}" title="{numero}" type="checkbox" value="{valor}"/></td></tr>"#,
            marcado = if i == 0 { "checked=\"checked\"" } else { "" },
            numero = escapar(&d.numero),
            valor = escapar(&d.id_documento),
        ).expect("escrever em String");
    }
    let primeiro = escapar(&documentos[0].id_documento);
    Resposta::html(format!(
        r#"<html><body>
<form id="frmBlocoEscolher" method="post" action="controlador.php?acao=bloco_escolher&acao_origem=bloco_escolher&arvore=1&id_procedimento={p}&id_documento={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH}">
<select id="selBloco" name="selBloco" class="infraSelect">
{opcoes}</select>
<button type="submit" name="sbmIncluir" id="sbmIncluir" value="Incluir">Incluir</button>
<table>{caixas}</table>
<input type="hidden" id="hdnDocumentosNroItens" name="hdnDocumentosNroItens" value="{quantos}" />
<input type="hidden" id="hdnDocumentosItens" name="hdnDocumentosItens" value="{ids}" />
<input type="hidden" id="hdnDocumentosItensSelecionados" name="hdnDocumentosItensSelecionados" value="{primeiro}" />
</form></body></html>"#,
        quantos = documentos.len(),
    ))
}
