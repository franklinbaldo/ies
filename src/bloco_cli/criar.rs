//! Prévia Web de criação de bloco de assinatura; nenhuma submissão.
use super::{EnvioDoControle, EnvioHttps, FalhaLeitura, ReferenciasDaSessao};
use crate::Cli;
use crate::acao_do_sei::AcaoDoSei;
use crate::bloco::{self, TipoBloco};
use crate::formulario_web::localizar_formulario;
use crate::tela_html::{HtmlServido, arvore};
use crate::transporte::{self, envio_post::ler_tela_via};

pub(crate) fn link_novo(html: &HtmlServido) -> Result<String, FalhaLeitura> {
    let dom = arvore(html);
    let mut alvo = None;
    // A lista UnB do Todos publica o mesmo btnNovo nas duas barras.
    for botao in dom.com_id("btnNovo") {
        let url = botao
            .atributo("onclick")
            .and_then(crate::sessao_login::tela::url_do_onclick)
            .ok_or(FalhaLeitura::HtmlDeBlocosInvalido)?;
        if alvo.as_deref().is_some_and(|anterior| anterior != url) {
            return Err(FalhaLeitura::HtmlDeBlocosInvalido);
        }
        alvo = Some(url.to_owned());
    }
    alvo.ok_or(FalhaLeitura::HtmlDeBlocosInvalido)
}

pub(crate) fn campo_descricao(html: &HtmlServido) -> Result<&'static str, FalhaLeitura> {
    let dom = arvore(html);
    let form = localizar_formulario(&dom, "frmBlocoCadastro")?;
    let mut campos = form
        .elementos("textarea")
        .filter_map(|no| match no.atributo("name") {
            Some("txtDescricao") if !no.presente("disabled") => Some("txtDescricao"),
            Some("txaDescricao") if !no.presente("disabled") => Some("txaDescricao"),
            _ => None,
        });
    let campo = campos.next().ok_or(FalhaLeitura::HtmlDeBlocosInvalido)?;
    if campos.next().is_some()
        || !form.descendentes().any(|no| {
            no.atributo("name") == Some("sbmCadastrarBloco") && !no.presente("disabled")
        })
    {
        return Err(FalhaLeitura::HtmlDeBlocosInvalido);
    }
    Ok(campo)
}

fn previa_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    descricao: &str,
    envio: &mut V,
) -> Result<serde_json::Value, FalhaLeitura> {
    let controle = ler_tela_via(
        sessao.url_do_controle(),
        sessao.cookie(),
        sessao.origem(),
        |url, cookie| envio.get(url, cookie),
    )?;
    let unidade = crate::controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade_esperada
        .is_some_and(|v| v.split_whitespace().collect::<Vec<_>>().join(" ") != unidade)
    {
        return Err(FalhaLeitura::UnidadeInesperada);
    }
    let url_lista = bloco::link_do_menu(
        &controle.instalacao,
        &controle.url,
        &controle.html,
        TipoBloco::Assinatura,
    )?;
    let cookie = sessao.cookie().valor()?;
    let lista = transporte::decodificar_resposta(envio.get(&url_lista, cookie)?)?;
    let url = bloco::resolver_link(
        &controle.instalacao,
        &url_lista,
        &link_novo(&lista)?,
        AcaoDoSei::BlocoAssinaturaCadastrar,
        None,
    )?;
    let cadastro = transporte::decodificar_resposta(envio.get(&url, cookie)?)?;
    campo_descricao(&cadastro)?;
    Ok(serde_json::json!({
        "unidade": unidade,
        "tipo_bloco": TipoBloco::Assinatura,
        "descricao": descricao,
        "confirmado": false,
        "alterado": false,
        "bloco": null
    }))
}

pub(super) fn executar(cli: &Cli, descricao: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!descricao.trim().is_empty(), "descrição do bloco vazia");
    let arquivos = crate::configuracao::arquivos_de_acesso()?;
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let previa = previa_via(&sessao, cli.unidade.as_deref(), descricao, &mut EnvioHttps)?;
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&previa)?);
    } else {
        println!("IES: prévia de novo bloco de assinatura: {descricao}. Nada gravado.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn novo_duplicado_so_quando_mesmo_destino() {
        let botao = "<button id='btnNovo' onclick=\"location.href='controlador.php?acao=bloco_assinatura_cadastrar'\">Novo</button>";
        let html = HtmlServido::de_teste(&format!("{botao}{botao}"));
        assert_eq!(
            link_novo(&html).unwrap(),
            "controlador.php?acao=bloco_assinatura_cadastrar"
        );
        let outro = botao.replace("cadastrar", "listar");
        let html = HtmlServido::de_teste(&format!("{botao}{outro}"));
        assert!(link_novo(&html).is_err());
        assert!(link_novo(&HtmlServido::de_teste("<html></html>")).is_err());
    }
}
