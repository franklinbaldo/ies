//! Cadastro somente leitura a partir de `documento_alterar` publicado na barra
//! (Todos `tests/paginas_reais/documento_alterar.html`, SEI 4.0.9; Refs #60).
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{barra_do_no, endereco::conferir_endereco_relativo_por};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{atributos, nos_com_id, nos_com_id_em};
use crate::instalacao::Instalacao;
use serde::Serialize;
type Tag<'p> = &'p tl::HTMLTag<'p>;
#[path = "propriedades_documento/anexo.rs"]
mod anexo;
pub(crate) use anexo::ler_anexo_publicado;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PropriedadesDocumento {
    NaoPedidas,
    NaoPublicadas,
    Lidas(PropriedadesLidas),
}

/// Destino e ação de cadastro que a própria barra do SEI publicou.
/// Não derivar o cadastro por URL criada a partir do id.
pub(crate) struct LinkDoCadastro {
    pub(crate) url: String,
    pub(crate) fonte: AcaoDoSei,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct PropriedadesLidas {
    pub(crate) fonte: AcaoDoSei,
    pub(crate) tipo: String,
    pub(crate) descricao: String,
    pub(crate) nome_na_arvore: String,
    pub(crate) observacoes: String,
    pub(crate) nivel_de_acesso: Option<Nivel>,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Nivel {
    Publico,
    Restrito,
    Sigiloso,
}

/// O cadastro do documento (barra, formulário, anexo) fora da forma observada:
/// um erro por fato.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroPropriedadesDocumento {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    /// `EnderecoDoConteudo::do_cadastro` recusou o link do anexo.
    LinkAnexoCadastroInvalido,
    AnexoDoCadastroDivergente,
    BarraCadastroDocumentoIlegivel,
    CadastroDocumentoIlegivel,
    CampoCadastroDocumentoAusente,
    CampoCadastroDocumentoRepetido,
    FormularioCadastroDocumentoAusente,
    FormulariosCadastroDocumentoRepetidos,
    IdentidadeCadastroDocumentoDivergente,
    LinkCadastroDocumentoInvalido,
    LinksAnexoCadastroRepetidos,
    LinksCadastroDocumentoRepetidos,
    NivelCadastroDocumentoInesperado,
    TextoCadastroDocumentoIlegivel,
    TipoCadastroDocumentoRepetido,
    TipoCadastroDocumentoVazio,
    ValorCadastroDocumentoAusente,
    NivelCadastroDocumentoIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
}

fn atributo(tag: Tag<'_>, nome: &'static str) -> Result<Option<String>, ErroPropriedadesDocumento> {
    let a = atributos(tag).map_err(ErroPropriedadesDocumento::Atributo)?;
    a.valor(nome)
        .map_err(ErroPropriedadesDocumento::Atributo)?
        .map(|v| {
            decodificar_atributo(v).ok_or(ErroPropriedadesDocumento::CadastroDocumentoIlegivel)
        })
        .transpose()
}
fn campo<'p>(
    form: Tag<'p>,
    parser: &'p tl::Parser<'p>,
    id: &str,
) -> Result<Tag<'p>, ErroPropriedadesDocumento> {
    let nos = nos_com_id_em(form, parser, id)
        .map_err(|_| ErroPropriedadesDocumento::CadastroDocumentoIlegivel)?;
    match nos.as_slice() {
        [unico] => unico
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroPropriedadesDocumento::CadastroDocumentoIlegivel),
        [] => Err(ErroPropriedadesDocumento::CampoCadastroDocumentoAusente),
        _ => Err(ErroPropriedadesDocumento::CampoCadastroDocumentoRepetido),
    }
}
fn valor(tag: Tag<'_>) -> Result<String, ErroPropriedadesDocumento> {
    atributo(tag, "value")?.ok_or(ErroPropriedadesDocumento::ValorCadastroDocumentoAusente)
}
fn texto(tag: Tag<'_>, parser: &tl::Parser<'_>) -> Result<String, ErroPropriedadesDocumento> {
    decodificar_atributo(&tag.inner_text(parser))
        .ok_or(ErroPropriedadesDocumento::TextoCadastroDocumentoIlegivel)
}
fn nivel(
    form: Tag<'_>,
    parser: &tl::Parser<'_>,
) -> Result<Option<Nivel>, ErroPropriedadesDocumento> {
    let mut marcados = Vec::new();
    for no in form.query_selector(parser, "input").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let a =
            atributos(tag).map_err(ErroPropriedadesDocumento::NivelCadastroDocumentoIlegivel)?;
        if a.valor("name")
            .map_err(ErroPropriedadesDocumento::NivelCadastroDocumentoIlegivel)?
            == Some("rdoNivelAcesso")
            && a.presente("checked")
        {
            marcados.push(atributo(tag, "value")?);
        }
    }
    match marcados.as_slice() {
        [] => Ok(None),
        [Some(x)] if x == "0" => Ok(Some(Nivel::Publico)),
        [Some(x)] if x == "1" => Ok(Some(Nivel::Restrito)),
        [Some(x)] if x == "2" => Ok(Some(Nivel::Sigiloso)),
        _ => Err(ErroPropriedadesDocumento::NivelCadastroDocumentoInesperado),
    }
}
/// Usa somente o link oferecido pelo SEI, sem inventar URLs assinadas.
pub(crate) fn ler_link(
    instalacao: &Instalacao,
    arvore: &str,
    documento: &str,
    processo: &str,
) -> Result<Option<LinkDoCadastro>, ErroPropriedadesDocumento> {
    let Some(barra) = barra_do_no(arvore, documento)
        .map_err(|_| ErroPropriedadesDocumento::BarraCadastroDocumentoIlegivel)?
    else {
        return Ok(None);
    };
    let dom = tl::parse(&barra, tl::ParserOptions::default())
        .map_err(|_| ErroPropriedadesDocumento::BarraCadastroDocumentoIlegivel)?;
    let mut encontrado = None;
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(dom.parser()).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(href) = atributo(tag, "href")? else {
            continue;
        };
        let acao = href
            .strip_prefix("controlador.php?")
            .and_then(|q| q.split('&').find_map(|p| p.strip_prefix("acao=")));
        // UFRJ 4.1: processo fechado oferece consultar em lugar de alterar;
        // externos publicam as ações `_recebido` (cassette UFRJ #60).
        let Some(
            fonte @ (AcaoDoSei::DocumentoAlterar
            | AcaoDoSei::DocumentoConsultar
            | AcaoDoSei::DocumentoAlterarRecebido
            | AcaoDoSei::DocumentoConsultarRecebido),
        ) = acao.and_then(AcaoDoSei::ler)
        else {
            continue;
        };
        conferir_endereco_relativo_por(instalacao, &href, fonte, "id_procedimento", processo)
            .map_err(|_| ErroPropriedadesDocumento::LinkCadastroDocumentoInvalido)?;
        let url =
            conferir_endereco_relativo_por(instalacao, &href, fonte, "id_documento", documento)
                .map_err(|_| ErroPropriedadesDocumento::LinkCadastroDocumentoInvalido)?;
        if encontrado.replace(LinkDoCadastro { url, fonte }).is_some() {
            return Err(ErroPropriedadesDocumento::LinksCadastroDocumentoRepetidos);
        }
    }
    Ok(encontrado)
}
/// Lê o cadastro pelo formulário servido, nunca o submete.
pub(crate) fn ler_formulario(
    html: &str,
    documento: &str,
    processo: &str,
) -> Result<PropriedadesLidas, ErroPropriedadesDocumento> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroPropriedadesDocumento::CadastroDocumentoIlegivel)?;
    let parser = dom.parser();
    let nos = nos_com_id(&dom, "frmDocumentoCadastro")
        .map_err(|_| ErroPropriedadesDocumento::CadastroDocumentoIlegivel)?;
    let form = match nos.as_slice() {
        [unico] => unico
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroPropriedadesDocumento::CadastroDocumentoIlegivel)?,
        [] => return Err(ErroPropriedadesDocumento::FormularioCadastroDocumentoAusente),
        _ => {
            return Err(ErroPropriedadesDocumento::FormulariosCadastroDocumentoRepetidos);
        }
    };
    if valor(campo(form, parser, "hdnIdDocumento")?)? != documento
        || valor(campo(form, parser, "hdnIdProcedimento")?)? != processo
    {
        return Err(ErroPropriedadesDocumento::IdentidadeCadastroDocumentoDivergente);
    }
    let tipo = texto(campo(form, parser, "lblSerieTitulo")?, parser)?;
    let tipo = if tipo.trim().is_empty() {
        // Cadastro do externo no SEI 4.0.9: o rótulo é vazio, mas o tipo
        // efetivamente escolhido está no option selecionado de selSerie.
        let select = campo(form, parser, "selSerie")?;
        let mut selecionados = Vec::new();
        for no in select
            .query_selector(parser, "option")
            .into_iter()
            .flatten()
        {
            let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            let a = atributos(tag).map_err(ErroPropriedadesDocumento::Atributo)?;
            if a.presente("selected") {
                selecionados.push(texto(tag, parser)?);
            }
        }
        match selecionados.as_slice() {
            [unico] if !unico.trim().is_empty() => unico.clone(),
            [] | [_] => return Err(ErroPropriedadesDocumento::TipoCadastroDocumentoVazio),
            _ => return Err(ErroPropriedadesDocumento::TipoCadastroDocumentoRepetido),
        }
    } else {
        tipo
    };
    Ok(PropriedadesLidas {
        fonte: AcaoDoSei::DocumentoAlterar,
        tipo,
        descricao: valor(campo(form, parser, "txtDescricao")?)?,
        nome_na_arvore: valor(campo(form, parser, "txtNomeArvore")?)?,
        observacoes: texto(campo(form, parser, "txaObservacoes")?, parser)?,
        nivel_de_acesso: nivel(form, parser)?,
    })
}

pub(crate) fn linha_humana(props: &PropriedadesDocumento) -> Option<String> {
    match props {
        PropriedadesDocumento::NaoPedidas => None,
        PropriedadesDocumento::NaoPublicadas => Some("Cadastro não publicado.".to_owned()),
        PropriedadesDocumento::Lidas(p) => {
            let nivel = match &p.nivel_de_acesso {
                Some(Nivel::Publico) => "público",
                Some(Nivel::Restrito) => "restrito",
                Some(Nivel::Sigiloso) => "sigiloso",
                None => "não informado",
            };
            Some(format!(
                "Cadastro: tipo {}; descrição {}; nome na árvore {}; observações {}; acesso {}.",
                p.tipo, p.descricao, p.nome_na_arvore, p.observacoes, nivel
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
    use crate::arvore_processo::tests_barra_documento::{barra, com_barra};
    use crate::arvore_processo::tests_tela::instalacao;
    #[test]
    fn dado_barra_servida_quando_lida_entao_confere_ambos_ids() {
        let html = arvore(&[
            no_processo(0, "10", NUP),
            no_documento(1, "21", "10", "0000101"),
        ]);
        let html = com_barra(&html, 1, &barra("10", "21", None));
        assert!(
            ler_link(&instalacao(), &html, "21", "10")
                .unwrap()
                .is_some()
        );
        assert!(
            ler_link(&instalacao(), &html, "22", "10")
                .unwrap()
                .is_none()
        );
        assert!(ler_link(&instalacao(), &html, "21", "11").is_err());
    }
    #[test]
    fn dado_nativo_assinado_em_processo_fechado_quando_leio_entao_consulto_cadastro() {
        // Dado UFRJ 4.1: em processo fechado documento_consultar substitui alterar.
        // Quando executo ies documento DOC --com-propriedades,
        // Então sigo somente o link da barra, sem fabricar URL de alteração.
        let html = arvore(&[
            no_processo(0, "10", NUP),
            no_documento(1, "21", "10", "0000101"),
        ]);
        let barra =
            barra("10", "21", None).replace("acao=documento_alterar", "acao=documento_consultar");
        let html = com_barra(&html, 1, &barra);
        let link = ler_link(&instalacao(), &html, "21", "10")
            .unwrap()
            .expect("consulta publicada na barra");
        assert!(link.url.contains("acao=documento_consultar&"));
    }

    #[test]
    fn dado_externo_fechado_quando_leio_entao_consulto_cadastro_recebido() {
        // Dado UFRJ 4.1: externo oferece documento_consultar_recebido.
        // Quando peço --com-propriedades, então a URL da barra é utilizada.
        let html = arvore(&[
            no_processo(0, "10", NUP),
            no_documento(1, "21", "10", "0000101"),
        ]);
        let barra = barra("10", "21", None).replace(
            "acao=documento_alterar",
            "acao=documento_consultar_recebido",
        );
        let html = com_barra(&html, 1, &barra);
        let link = ler_link(&instalacao(), &html, "21", "10")
            .unwrap()
            .expect("consulta do externo publicada na barra");
        assert!(link.url.contains("acao=documento_consultar_recebido&"));
    }

    #[test]
    fn dado_externo_editavel_quando_leio_entao_uso_alteracao_recebida() {
        // Dado um externo com documento_alterar_recebido observado,
        // Quando executo --com-propriedades, então leio o link publicado.
        let html = arvore(&[
            no_processo(0, "10", NUP),
            no_documento(1, "21", "10", "0000101"),
        ]);
        let barra = barra("10", "21", None)
            .replace("acao=documento_alterar", "acao=documento_alterar_recebido");
        let html = com_barra(&html, 1, &barra);
        let link = ler_link(&instalacao(), &html, "21", "10")
            .unwrap()
            .expect("alteração do externo publicada na barra");
        assert!(link.url.contains("acao=documento_alterar_recebido&"));
    }

    #[test]
    fn dado_cadastro_externo_com_titulo_vazio_quando_leio_entao_uso_tipo_selecionado() {
        // Dado cadastro externo observado em Todos
        // tests/paginas_reais/documento_alterar_recebido.html @main,
        // com lblSerieTitulo vazio e selSerie selecionando Anexo,
        // quando executo ies documento DOC --com-propriedades --json,
        // entao o tipo publicado e Anexo, nao tipo_cadastro_documento_vazio.
        let html = r#"<form id="frmDocumentoCadastro">
            <input id="hdnIdDocumento" value="21">
            <input id="hdnIdProcedimento" value="10">
            <label id="lblSerieTitulo"></label>
            <select id="selSerie">
                <option value="null">&nbsp;</option>
                <option value="293" selected="selected">Anexo</option>
                <option value="290">Ofício</option>
            </select>
            <input id="txtDescricao" value="Backup de teste">
            <input id="txtNomeArvore" value="Anexo externo">
            <textarea id="txaObservacoes"></textarea>
            <input name="rdoNivelAcesso" value="0" checked>
        </form>"#;
        let lida = ler_formulario(html, "21", "10").unwrap();
        assert_eq!(lida.tipo, "Anexo");
        assert_eq!(lida.descricao, "Backup de teste");
        assert_eq!(lida.nivel_de_acesso, Some(Nivel::Publico));
    }

    #[test]
    fn dado_formulario_observado_quando_lido_entao_preserva_campos() {
        // Caso Todos tests/paginas_reais/documento_alterar.html @323eb01.
        let html = r#"<form id="frmDocumentoCadastro" method="post">
            <input id="hdnIdDocumento" value="21"><input id="hdnIdProcedimento" value="10">
            <label id="lblSerieTitulo">Ofício</label>
            <input id="txtDescricao" value="Revisão">
            <input id="txtNomeArvore" value="Ofício 0000101">
            <textarea id="txaObservacoes">texto livre</textarea>
            <input name="rdoNivelAcesso" value="0" checked>
            <input name="rdoNivelAcesso" value="1"></form>"#;
        let lida = ler_formulario(html, "21", "10").unwrap();
        assert_eq!(
            (lida.tipo.as_str(), lida.descricao.as_str()),
            ("Ofício", "Revisão")
        );
        assert_eq!(lida.nivel_de_acesso, Some(Nivel::Publico));
        assert!(ler_formulario(html, "21", "11").is_err());
    }
}
