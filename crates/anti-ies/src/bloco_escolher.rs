//! Tela `bloco_escolher` ("Incluir em Bloco de Assinatura", #381) gerada do
//! SQLite: `frmBlocoEscolher` com `selBloco` (blocos de assinatura da
//! unidade) e a caixa do documento, na forma do card
//! `bloco-incluir-anotar-retirar-unb`. Hipótese sintética; o POST de
//! inclusão é a próxima fatia (BDD-381-02) e aqui é recusado.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::blocos::{self, Tipo};
use iesei::documento_versoes;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

const HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
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
    Resposta::html(format!(
        r#"<html><body>
<form id="frmBlocoEscolher" method="post" action="controlador.php?acao=bloco_escolher&acao_origem=bloco_escolher&arvore=1&id_procedimento={p}&id_documento={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH}">
<select id="selBloco" name="selBloco" class="infraSelect">
{opcoes}</select>
<button type="submit" name="sbmIncluir" id="sbmIncluir" value="Incluir">Incluir</button>
<table><tr><td><input checked="checked" class="infraCheckbox" id="chkDocumentosItem0" name="chkDocumentosItem0" title="{numero}" type="checkbox" value="{id}"/></td></tr></table>
<input type="hidden" id="hdnDocumentosNroItens" name="hdnDocumentosNroItens" value="1" />
<input type="hidden" id="hdnDocumentosItens" name="hdnDocumentosItens" value="{id}" />
<input type="hidden" id="hdnDocumentosItensSelecionados" name="hdnDocumentosItensSelecionados" value="{id}" />
</form></body></html>"#,
        numero = escapar(&documento.numero),
    ))
}
