//! Cadastro somente leitura a partir de `documento_alterar` publicado na barra
//! (Todos `tests/paginas_reais/documento_alterar.html`, SEI 4.0.9; Refs #60).
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{barra_do_no, endereco::conferir_endereco_relativo_por};
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::instalacao::Instalacao;
use serde::Serialize;
type Tag<'p> = crate::tela_html::Elemento<'p>;
#[path = "propriedades_documento/anexo.rs"]
mod anexo;
pub(crate) use anexo::ler_anexo_publicado;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PropriedadesDocumento {
    NaoPedidas,
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
    pub(crate) nivel_de_acesso: crate::dominio::NivelDeAcesso,
}

/// O cadastro do documento (barra, formulário, anexo) fora da forma observada:
/// um erro por fato.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroPropriedadesDocumento {
    /// Nenhum `rdoNivelAcesso` marcado no cadastro do documento.
    NivelDeAcessoNaoMarcado,
    /// `EnderecoDoConteudo::do_cadastro` recusou o link do anexo.
    LinkAnexoCadastroInvalido(#[source] crate::arvore_processo::ErroConteudoNaArvore),
    AnexoDoCadastroDivergente,
    BarraCadastroDocumentoIlegivel(#[source] crate::arvore_processo::ErroBarraDoDocumento),
    CampoCadastroDocumentoAusente,
    CampoCadastroDocumentoRepetido,
    FormularioCadastroDocumentoAusente,
    FormulariosCadastroDocumentoRepetidos,
    IdentidadeCadastroDocumentoDivergente,
    LinkCadastroDocumentoInvalido(crate::arvore_processo::endereco::EnderecoRecusado),
    LinksAnexoCadastroRepetidos,
    LinksCadastroDocumentoRepetidos,
    NivelCadastroDocumentoInesperado,
    TipoCadastroDocumentoRepetido,
    TipoCadastroDocumentoVazio,
    ValorCadastroDocumentoAusente,
}

fn campo<'p>(form: Tag<'p>, id: &str) -> Result<Tag<'p>, ErroPropriedadesDocumento> {
    let nos = form.com_id(id).collect::<Vec<_>>();
    match nos.as_slice() {
        [unico] => Ok(*unico),
        [] => Err(ErroPropriedadesDocumento::CampoCadastroDocumentoAusente),
        _ => Err(ErroPropriedadesDocumento::CampoCadastroDocumentoRepetido),
    }
}
fn valor(tag: Tag<'_>) -> Result<String, ErroPropriedadesDocumento> {
    tag.atributo("value")
        .map(str::to_owned)
        .ok_or(ErroPropriedadesDocumento::ValorCadastroDocumentoAusente)
}
fn texto(tag: Tag<'_>) -> String {
    tag.texto()
}
fn nivel(form: Tag<'_>) -> Result<crate::dominio::NivelDeAcesso, ErroPropriedadesDocumento> {
    let mut marcados = Vec::new();
    for no in form.elementos("input") {
        if no.atributo("name") == Some("rdoNivelAcesso") && no.presente("checked") {
            marcados.push(no.atributo("value").map(str::to_owned));
        }
    }
    match marcados.as_slice() {
        [] => Err(ErroPropriedadesDocumento::NivelDeAcessoNaoMarcado),
        [Some(x)] if x == "0" => Ok(crate::dominio::NivelDeAcesso::Publico),
        [Some(x)] if x == "1" => Ok(crate::dominio::NivelDeAcesso::Restrito),
        [Some(x)] if x == "2" => Ok(crate::dominio::NivelDeAcesso::Sigiloso),
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
        .map_err(ErroPropriedadesDocumento::BarraCadastroDocumentoIlegivel)?
    else {
        return Ok(None);
    };
    let dom = crate::tela_html::arvore(&barra);
    let mut encontrado = None;
    for no in dom.elementos("a") {
        let Some(href) = no.atributo("href").map(str::to_owned) else {
            continue;
        };
        let acao = href
            .strip_prefix("controlador.php?")
            .and_then(|q| ConsultaDoControlador::ler(q).ok())
            .and_then(|q| q.acao().ok())
            .and_then(AcaoDoSei::ler);
        // UFRJ 4.1: processo fechado oferece consultar em lugar de alterar;
        // externos publicam as ações `_recebido` (cassette UFRJ #60).
        let Some(
            fonte @ (AcaoDoSei::DocumentoAlterar
            | AcaoDoSei::DocumentoConsultar
            | AcaoDoSei::DocumentoAlterarRecebido
            | AcaoDoSei::DocumentoConsultarRecebido),
        ) = acao
        else {
            continue;
        };
        conferir_endereco_relativo_por(instalacao, &href, fonte, "id_procedimento", processo)
            .map_err(ErroPropriedadesDocumento::LinkCadastroDocumentoInvalido)?;
        let url =
            conferir_endereco_relativo_por(instalacao, &href, fonte, "id_documento", documento)
                .map_err(ErroPropriedadesDocumento::LinkCadastroDocumentoInvalido)?;
        if encontrado.replace(LinkDoCadastro { url, fonte }).is_some() {
            return Err(ErroPropriedadesDocumento::LinksCadastroDocumentoRepetidos);
        }
    }
    Ok(encontrado)
}
/// Lê o cadastro pelo formulário servido, nunca o submete.
/// O `frmDocumentoCadastro` único da tela, conferido como o cadastro do
/// `documento` no `processo` pedidos (`hdnIdDocumento`, `hdnIdProcedimento`).
fn cadastro_do_documento<'p>(
    dom: &'p crate::tela_html::Documento,
    documento: &str,
    processo: &str,
) -> Result<Tag<'p>, ErroPropriedadesDocumento> {
    let nos = dom.com_id("frmDocumentoCadastro").collect::<Vec<_>>();
    let form = match nos.as_slice() {
        [unico] => *unico,
        [] => return Err(ErroPropriedadesDocumento::FormularioCadastroDocumentoAusente),
        _ => {
            return Err(ErroPropriedadesDocumento::FormulariosCadastroDocumentoRepetidos);
        }
    };
    if valor(campo(form, "hdnIdDocumento")?)? != documento
        || valor(campo(form, "hdnIdProcedimento")?)? != processo
    {
        return Err(ErroPropriedadesDocumento::IdentidadeCadastroDocumentoDivergente);
    }
    Ok(form)
}

pub(crate) fn ler_formulario(
    html: &crate::tela_html::HtmlServido,
    documento: &str,
    processo: &str,
) -> Result<PropriedadesLidas, ErroPropriedadesDocumento> {
    let dom = crate::tela_html::arvore(html);
    let form = cadastro_do_documento(&dom, documento, processo)?;
    let tipo = texto(campo(form, "lblSerieTitulo")?);
    let tipo = if tipo.trim().is_empty() {
        // Cadastro do externo no SEI 4.0.9: o rótulo é vazio, mas o tipo
        // efetivamente escolhido está no option selecionado de selSerie.
        let select = campo(form, "selSerie")?;
        let mut selecionados = Vec::new();
        for no in select.elementos("option") {
            if no.presente("selected") {
                selecionados.push(texto(no));
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
        descricao: valor(campo(form, "txtDescricao")?)?,
        nome_na_arvore: valor(campo(form, "txtNomeArvore")?)?,
        observacoes: texto(campo(form, "txaObservacoes")?),
        nivel_de_acesso: nivel(form)?,
    })
}

pub(crate) fn linha_humana(props: &PropriedadesDocumento) -> Option<String> {
    match props {
        PropriedadesDocumento::NaoPedidas => None,
        PropriedadesDocumento::Lidas(p) => {
            let nivel = match &p.nivel_de_acesso {
                crate::dominio::NivelDeAcesso::Publico => "público",
                crate::dominio::NivelDeAcesso::Restrito => "restrito",
                crate::dominio::NivelDeAcesso::Sigiloso => "sigiloso",
            };
            Some(format!(
                "Cadastro: tipo {}; descrição {}; nome na árvore {}; observações {}; acesso {}.",
                p.tipo, p.descricao, p.nome_na_arvore, p.observacoes, nivel
            ))
        }
    }
}

#[cfg(test)]
mod acervo;
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
        let html = crate::tela_html::HtmlServido::de_teste(
            r#"<form id="frmDocumentoCadastro">
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
        </form>"#,
        );
        let lida = ler_formulario(&html, "21", "10").unwrap();
        assert_eq!(lida.tipo, "Anexo");
        assert_eq!(lida.descricao, "Backup de teste");
        assert_eq!(lida.nivel_de_acesso, crate::dominio::NivelDeAcesso::Publico);
    }

    #[test]
    fn dado_formulario_observado_quando_lido_entao_preserva_campos() {
        // Caso Todos tests/paginas_reais/documento_alterar.html @323eb01.
        let html = crate::tela_html::HtmlServido::de_teste(
            r#"<form id="frmDocumentoCadastro" method="post">
            <input id="hdnIdDocumento" value="21"><input id="hdnIdProcedimento" value="10">
            <label id="lblSerieTitulo">Ofício</label>
            <input id="txtDescricao" value="Revisão">
            <input id="txtNomeArvore" value="Ofício 0000101">
            <textarea id="txaObservacoes">texto livre</textarea>
            <input name="rdoNivelAcesso" value="0" checked>
            <input name="rdoNivelAcesso" value="1"></form>"#,
        );
        let lida = ler_formulario(&html, "21", "10").unwrap();
        assert_eq!(
            (lida.tipo.as_str(), lida.descricao.as_str()),
            ("Ofício", "Revisão")
        );
        assert_eq!(lida.nivel_de_acesso, crate::dominio::NivelDeAcesso::Publico);
        assert!(ler_formulario(&html, "21", "11").is_err());
    }
}
