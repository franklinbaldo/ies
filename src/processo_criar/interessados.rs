//! Interessados do cadastro de processo: autocomplete que a tela publicou.
//! Cada pedido faz uma consulta sem efeito de escrita, por URL assinada
//! observada, e exige identidade unívoca; nenhum nome vira ID inventado.
use crate::formulario_web::{
    ConsultaAjax, EfeitoAvaliado, TelaServida, derivar_consulta_ajax, localizar_formulario,
    decodificar_atributo,
};
use crate::html_atributos::{valor_do_atributo, atributo_presente};
use crate::instalacao::Instalacao;
use crate::transporte::{ler_limitado, corpo::decode_http_body};
use crate::visao_do_controle::EnvioDoControle;

const PREFIXO: &str = "objAutoCompletarInteressadoRI1225 = new infraAjaxAutoCompletar('hdnIdInteressadoProcedimento','txtInteressadoProcedimento','";
const AUTOCOMPLETAR: ConsultaAjax = ConsultaAjax {
    acao_ajax: "contato_auto_completar_contexto_RI1225",
    campos: &["palavras_pesquisa"],
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "docs/como-o-sei-funciona/inicio-do-processo.md: autocomplete do contato",
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErroInteressado {
    ControlesAusentes,
    AutocompletarNaoDeclarado,
    DerivacaoRecusada,
    RespostaInesperada,
    NaoOferecido,
    Ambiguo,
}

impl ErroInteressado {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::ControlesAusentes => "interessado_controles_ausentes",
            Self::AutocompletarNaoDeclarado => "interessado_autocompletar_nao_declarado",
            Self::DerivacaoRecusada => "interessado_consulta_nao_derivada",
            Self::RespostaInesperada => "interessado_resposta_nao_reconhecida",
            Self::NaoOferecido => "interessado_de_processo_nao_oferecido",
            Self::Ambiguo => "interessado_de_processo_ambiguo",
        }
    }
}

fn campo_unico(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    elemento: &str,
    nome: &str,
) -> Result<(), ErroInteressado> {
    let mut achados = 0;
    for handle in form.query_selector(parser, elemento).into_iter().flatten() {
        let tag = handle.get(parser).and_then(tl::Node::as_tag)
            .ok_or(ErroInteressado::ControlesAusentes)?;
        let lido = valor_do_atributo(tag, "name")
            .map_err(|_| ErroInteressado::ControlesAusentes)?;
        if lido.as_deref() == Some(nome) {
            if atributo_presente(tag, "disabled")
                .map_err(|_| ErroInteressado::ControlesAusentes)?
            {
                return Err(ErroInteressado::ControlesAusentes);
            }
            achados += 1;
        }
    }
    if achados == 1 { Ok(()) } else { Err(ErroInteressado::ControlesAusentes) }
}

/// URL declarada na inicialização do autocomplete, sem executar o script.
/// Também confere os campos reais do formulário que receberá os IDs.
fn url_publicada(html: &str) -> Result<String, ErroInteressado> {
    use ErroInteressado as E;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| E::ControlesAusentes)?;
    let form = localizar_formulario(&dom, "frmProcedimentoCadastro")
        .map_err(|_| E::ControlesAusentes)?;
    campo_unico(form, dom.parser(), "input", "hdnInteressadosProcedimento")?;
    campo_unico(form, dom.parser(), "input", "txtInteressadoProcedimento")?;
    campo_unico(form, dom.parser(), "select", "selInteressadosProcedimento")?;
    let urls: Vec<&str> = html.lines().map(str::trim)
        .filter_map(|linha| linha.strip_prefix(PREFIXO))
        .filter_map(|resto| resto.strip_suffix("');"))
        .collect();
    match urls.as_slice() {
        [url] if url.starts_with("controlador_ajax.php?") => Ok((*url).to_owned()),
        _ => Err(E::AutocompletarNaoDeclarado),
    }
}

fn itens_da_resposta(xml: &str) -> Result<Vec<(String, String)>, ErroInteressado> {
    use ErroInteressado as E;
    let dom = tl::parse(xml, tl::ParserOptions::default())
        .map_err(|_| E::RespostaInesperada)?;
    let parser = dom.parser();
    let roots = dom.query_selector("itens").map_or(0, Iterator::count);
    if roots != 1 { return Err(E::RespostaInesperada); }
    let mut itens = Vec::new();
    for handle in dom.query_selector("item").into_iter().flatten() {
        let tag = handle.get(parser).and_then(tl::Node::as_tag)
            .ok_or(E::RespostaInesperada)?;
        let ler = |nome| -> Result<String, E> {
            valor_do_atributo(tag, nome)
                .map_err(|_| E::RespostaInesperada)?
                .and_then(|s| decodificar_atributo(&s))
                .filter(|s| !s.trim().is_empty())
                .ok_or(E::RespostaInesperada)
        };
        itens.push((ler("id")?, ler("descricao")?));
        if itens.len() > 500 { return Err(E::RespostaInesperada); }
    }
    Ok(itens)
}

fn escolher(pedido: &str, itens: &[(String, String)])
    -> Result<(String, String), ErroInteressado>
{
    use ErroInteressado as E;
    let normalizar = |texto: &str| texto.split_whitespace().collect::<Vec<_>>()
        .join(" ").to_lowercase();
    let pedido = normalizar(pedido);
    let encontrados: Vec<_> = itens.iter().filter(|(_, nome)| normalizar(nome) == pedido)
        .collect();
    match encontrados.as_slice() {
        [(id, nome)] => Ok((id.clone(), nome.clone())),
        [] => Err(E::NaoOferecido),
        _ => Err(E::Ambiguo),
    }
}

/// Resolve um interessado por POST de consulta, sem retry e sem GET extra.
/// A resposta XML informa o ID, não o texto digitado pelo usuário.
pub(crate) fn resolver<V: EnvioDoControle>(
    envio: &mut V,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    cookie: &str,
    pedidos: &[String],
    requisicoes: &mut u32,
) -> Result<Vec<(String, String)>, ErroInteressado> {
    use ErroInteressado as E;
    if pedidos.is_empty() { return Ok(Vec::new()); }
    let url = url_publicada(tela.html)?;
    let mut resultados = Vec::new();
    for pedido in pedidos {
        let consulta = derivar_consulta_ajax(
            &AUTOCOMPLETAR, instalacao, tela, &url, &[("palavras_pesquisa", pedido)],
        ).map_err(|_| E::DerivacaoRecusada)?;
        *requisicoes += 1;
        let resposta = envio.post(consulta, cookie).map_err(|_| E::RespostaInesperada)?;
        if resposta.resposta.status_code != 200
            || !matches!(
                resposta.resposta.mime.split(';').next().unwrap_or_default().trim(),
                "application/xml" | "text/xml"
            )
        {
            return Err(E::RespostaInesperada);
        }
        let mime = resposta.resposta.mime.clone();
        let bytes = ler_limitado(resposta.resposta.corpo, crate::MAX_HTML_BYTES)
            .map_err(|_| E::RespostaInesperada)?;
        let xml = decode_http_body(&bytes, &mime).map_err(|_| E::RespostaInesperada)?;
        let escolhido = escolher(pedido, &itens_da_resposta(&xml)?)?;
        if !resultados.iter().any(|(id, _): &(String, String)| *id == escolhido.0) {
            resultados.push(escolhido);
        }
    }
    Ok(resultados)
}

pub(crate) fn valor_da_lupa(itens: &[(String, String)]) -> String {
    itens.iter().map(|(id, nome)| format!("{id}±{nome}"))
        .collect::<Vec<_>>().join("¥")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xml_sem_correspondencia_exata_eh_recusado() {
        let xml = "<itens><item id='1' descricao='Pessoa A'/><item id='2' descricao='Pessoa B'/></itens>";
        let itens = itens_da_resposta(xml).unwrap();
        assert_eq!(escolher("Pessoa C", &itens), Err(ErroInteressado::NaoOferecido));
        assert_eq!(escolher("Pessoa A", &itens), Ok(("1".into(), "Pessoa A".into())));
    }
    #[test]
    fn duplicados_com_nome_igual_sao_ambiguos() {
        let xml = "<itens><item id='1' descricao='Pessoa A'/><item id='2' descricao='Pessoa A'/></itens>";
        assert_eq!(escolher("Pessoa A", &itens_da_resposta(xml).unwrap()), Err(ErroInteressado::Ambiguo));
    }
    #[test]
    fn codificacao_da_lupa_preserva_ids_e_nomes() {
        assert_eq!(valor_da_lupa(&[("1".into(), "Pessoa A".into()), ("2".into(), "Pessoa B".into())]), "1±Pessoa A¥2±Pessoa B");
    }
}
