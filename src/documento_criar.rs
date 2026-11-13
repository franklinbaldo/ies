//! Seleção, cadastro e reconciliação offline (#246); prévia Web no módulo fluxo.
#[cfg(test)]
mod acervo;
mod cadastro;
mod confirmacao;
pub(crate) use confirmacao::redirecionamento_do_mesmo_controlador;
mod fluxo;
mod guarda_navegacao;
mod guarda_script;
pub(crate) use cadastro::conferir_cadastro;
mod link_do_tipo;
pub(crate) use fluxo::{
    EstadoCriacao, PedidoCriacao, ResultadoCriacao, executar_criacao, validar_nivel_de_confirmacao,
};

use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::DocumentoNaArvore;
use crate::arvore_processo::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::instalacao::Instalacao;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCriar {
    ListaForaDaForma,
    TipoAusente,
    TipoAmbiguo,
    NaoAplicado,
    NaoConfirmado,
    LinkDoTipoNaoNavegavel,
    /// O link é literal, mas a função de navegação que ele chama não foi
    /// comprovada em script ativo.
    NavegacaoNaoComprovadaPorScript(#[source] guarda_navegacao::ErroGuardaDeNavegacao),
    EnderecoDoTipoNaoReconhecido(EnderecoRecusado),
    CadastroForaDaForma,
    TipoDoCadastroDiferente,
    DescricaoNaoOferecida,
    NivelNaoOferecido,
    CampoObrigatorioNaoAtendido,
}
/// Apenas nomes publicados em table#tblSeries; nunca interpreta JS como código.
pub(crate) fn tipos_oferecidos(
    html: &crate::tela_html::HtmlServido,
) -> Result<Vec<String>, ErroCriar> {
    let dom = crate::tela_html::arvore(html);
    let tabelas = dom.com_id("tblSeries").collect::<Vec<_>>();
    let [tabela] = tabelas.as_slice() else {
        return Err(ErroCriar::ListaForaDaForma);
    };
    let tabela = *tabela;
    if !tabela.nome().eq_ignore_ascii_case("table") {
        return Err(ErroCriar::ListaForaDaForma);
    }
    let mut tipos = Vec::new();
    for tag in tabela.descendentes() {
        if !tag.nome().eq_ignore_ascii_case("a") {
            continue;
        }
        let Some(href) = tag.atributo("href") else {
            continue;
        };
        let onclick = tag.atributo("onclick");
        // Na 4.0 "Externo" usa documento_receber, e não documento_gerar.
        // Nas 4.1/5.0, escolher(-1) também é externo.
        if !link_de_tipo_interno(href, onclick) {
            continue;
        }
        let nome = nome_visivel(&tag.texto());
        if !nome.is_empty() {
            tipos.push(nome);
        }
    }
    Ok(tipos)
}
/// Nome do tipo como o texto o publica, espaços colapsados. O U+00A0 fica:
/// o SEI 5 publica "Nome&nbsp;(Formulário)", e o nome segue com o espaço
/// inseparável, como se lia antes da #676.
fn nome_visivel(texto: &str) -> String {
    texto
        .split(|c: char| c.is_whitespace() && c != '\u{a0}')
        .filter(|parte| !parte.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn conferir_tipo(tipo: &str, oferecidos: &[String]) -> Result<(), ErroCriar> {
    match oferecidos.iter().filter(|s| s.as_str() == tipo).count() {
        0 => Err(ErroCriar::TipoAusente),
        1 => Ok(()),
        _ => Err(ErroCriar::TipoAmbiguo),
    }
}
/// Sucesso apenas com um id novo, número SEI e rótulo esperados, mantendo
/// exatamente todos os documentos anteriores. Falha pós-POST não é retry.
pub(crate) fn conferir_criacao(
    antes: &[DocumentoNaArvore],
    depois: &[DocumentoNaArvore],
    tipo: &str,
) -> Result<String, ErroCriar> {
    conferir_criacao_com_rotulo(antes, depois, |rotulo, numero| {
        // `Despacho 0012274` (UnB, PEN, UFRJ) ou, com numeração própria do
        // tipo, `Despacho 1407 (8446460)` (GO 5.0.3, live de 10/11/2026).
        rotulo == format!("{tipo} {numero}")
            || rotulo
                .strip_prefix(tipo)
                .and_then(|resto| resto.strip_prefix(' '))
                .and_then(|resto| resto.strip_suffix(&format!(" ({numero})")))
                .is_some_and(|serie| !serie.is_empty() && serie.bytes().all(|b| b.is_ascii_digit()))
    })
}

/// Como [`conferir_criacao`], com a conferência do rótulo que a árvore
/// publica para o documento novo (rótulo, número SEI). Externo na `UnB` (SEI 4.0.9): `Anexo (0011862)`.
pub(crate) fn conferir_criacao_com_rotulo(
    antes: &[DocumentoNaArvore],
    depois: &[DocumentoNaArvore],
    rotulo_confere: impl Fn(&str, &str) -> bool,
) -> Result<String, ErroCriar> {
    // Uma árvore inicial ambígua também inviabiliza provar unicidade:
    // não basta a leitura posterior ter IDs distintos.
    let mut originais = std::collections::HashSet::new();
    for doc in antes {
        if doc.id.is_empty() || !originais.insert(doc.id.as_str()) {
            return Err(ErroCriar::NaoConfirmado);
        }
    }
    let mut ids = std::collections::HashSet::new();
    for doc in depois {
        if doc.id.is_empty() || !ids.insert(doc.id.as_str()) {
            return Err(ErroCriar::NaoConfirmado);
        }
    }
    for original in antes {
        if !depois.iter().any(|doc| doc == original) {
            return Err(ErroCriar::NaoConfirmado);
        }
    }
    let novos: Vec<_> = depois
        .iter()
        .filter(|doc| !antes.iter().any(|d| d.id == doc.id))
        .collect();
    let novo = match novos.as_slice() {
        [] => return Err(ErroCriar::NaoAplicado),
        [doc] => *doc,
        _ => return Err(ErroCriar::NaoConfirmado),
    };
    let numero = novo
        .numero_sei
        .as_deref()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .ok_or(ErroCriar::NaoConfirmado)?;
    if !rotulo_confere(&novo.rotulo, numero)
        || antes
            .iter()
            .any(|d| d.numero_sei.as_deref() == Some(numero))
        || depois
            .iter()
            .filter(|d| d.numero_sei.as_deref() == Some(numero))
            .count()
            != 1
    {
        return Err(ErroCriar::NaoConfirmado);
    }
    Ok(numero.to_owned())
}

/// A 4.0 publica URL direta; a 4.1/5.0 publica escolher(id).
/// Nunca carrega credenciais de sessão nem URL assinada no Debug.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum EscolhaDoDocumento {
    Link(String),
    Formulario(String),
}

/// Não expõe URL assinada ou identificadores sensíveis no Debug.
impl std::fmt::Debug for EscolhaDoDocumento {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Link(_) => f.write_str("Link(<redigido>)"),
            Self::Formulario(_) => f.write_str("Formulario(<redigido>)"),
        }
    }
}

fn id_da_escolha_literal(onclick: &str) -> Option<String> {
    let chamada = onclick.trim().trim_end_matches(';').trim();
    let id = chamada.strip_prefix("escolher(")?.strip_suffix(')')?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then(|| id.to_owned())
}

/// A opçao externa nao integra a lista de tipos de documento nativo.
fn link_de_tipo_externo(href: &str, onclick: Option<&str>) -> bool {
    href.strip_prefix("controlador.php?")
        .and_then(|consulta| ConsultaDoControlador::ler(consulta).ok())
        .is_some_and(|consulta| consulta.acao() == Ok(AcaoDoSei::DocumentoReceber.como_na_url()))
        || onclick.is_some_and(|chamada| chamada.trim().trim_end_matches(';') == "escolher(-1)")
}

/// Reconhece tipos nativos sem tratar controles invalidos como externos.
fn link_de_tipo_interno(href: &str, onclick: Option<&str>) -> bool {
    !link_de_tipo_externo(href, onclick)
        && (href
            .strip_prefix("controlador.php?")
            .and_then(|consulta| ConsultaDoControlador::ler(consulta).ok())
            .is_some_and(|consulta| consulta.acao() == Ok(AcaoDoSei::DocumentoGerar.como_na_url()))
            || ((href == "#" || href == "javascript:void(0);")
                && onclick.and_then(id_da_escolha_literal).is_some()))
}

/// Reconhece só o corpo literal observado da função, com duas instruções
/// (reescrita do id e submit da lista). Texto de JS não é executado.
/// O parâmetro chama `id` (4.1) ou `idSerie` (SEI 5: PEN, UFRJ e GO).
fn script_prova_escolha(
    html: &crate::tela_html::HtmlServido,
) -> Result<(), guarda_navegacao::ErroGuardaDeNavegacao> {
    let corpo = |parametro: &str| {
        format!(
            "functionescolher({parametro}){{document.getElementById('hdnIdSerie').value={parametro};document.getElementById('frmDocumentoEscolherTipo').submit();}}"
        )
    };
    guarda_navegacao::script_publica_funcao_de_navegacao(html, "functionescolher(", &corpo("id"))
        .or_else(|_| {
            guarda_navegacao::script_publica_funcao_de_navegacao(
                html,
                "functionescolher(",
                &corpo("idSerie"),
            )
        })
}

/// A lista inicial pode mostrar apenas tipos recentes. O frontend 4.0/4.1
/// declara o botão "Exibir todos" por exibirSeries('T'), que altera apenas
/// o filtro da sessão. Nunca executar JS ou inferir a função pelo botão.
pub(crate) fn filtro_de_todos(html: &crate::tela_html::HtmlServido) -> Result<bool, ErroCriar> {
    let dom = crate::tela_html::arvore(html);
    let botoes = dom.com_id("ancExibirSeries").collect::<Vec<_>>();
    let [botao] = botoes.as_slice() else {
        return if botoes.is_empty() {
            Ok(false)
        } else {
            Err(ErroCriar::ListaForaDaForma)
        };
    };
    let tag = *botao;
    if !tag.nome().eq_ignore_ascii_case("a") {
        return Err(ErroCriar::ListaForaDaForma);
    }
    let href = tag
        .atributo("href")
        .map(str::to_owned)
        .ok_or(ErroCriar::LinkDoTipoNaoNavegavel)?;
    let onclick = tag
        .atributo("onclick")
        .map(str::to_owned)
        .ok_or(ErroCriar::LinkDoTipoNaoNavegavel)?;
    let chamada = onclick.trim().trim_end_matches(';').trim();
    // Filtro da sessão já em "todos": o botão volta a oferecer só os
    // utilizados, e a lista servida já está completa (UnB, 10/11/2026).
    if chamada == "exibirSeries('U')" || chamada == "exibirSeries(\"U\")" {
        return Ok(false);
    }
    if (href != "#" && href != "javascript:void(0);")
        || (chamada != "exibirSeries('T')" && chamada != "exibirSeries(\"T\")")
    {
        return Err(ErroCriar::LinkDoTipoNaoNavegavel);
    }
    guarda_navegacao::script_publica_funcao_de_navegacao(
        html,
        "functionexibirSeries(",
        "functionexibirSeries(tipo){document.getElementById('hdnFiltroSerie').value=tipo;document.getElementById('frmDocumentoEscolherTipo').submit();}",
    )
    .map_err(ErroCriar::NavegacaoNaoComprovadaPorScript)?;
    Ok(true)
}

/// Escolhe o tipo exato na lista observada, sem inventar URL ou executar JS.
/// O tipo externo (escolher(-1)) não é documento interno.
pub(crate) fn escolha_do_tipo(
    instalacao: &Instalacao,
    html: &crate::tela_html::HtmlServido,
    id_procedimento: &str,
    tipo: &str,
) -> Result<EscolhaDoDocumento, ErroCriar> {
    let dom = crate::tela_html::arvore(html);
    let tabelas = dom.com_id("tblSeries").collect::<Vec<_>>();
    let [tabela] = tabelas.as_slice() else {
        return Err(ErroCriar::ListaForaDaForma);
    };
    let tabela = *tabela;
    if !tabela.nome().eq_ignore_ascii_case("table") {
        return Err(ErroCriar::ListaForaDaForma);
    }
    let mut links = Vec::new();
    for tag in tabela.descendentes() {
        if !tag.nome().eq_ignore_ascii_case("a") {
            continue;
        }
        let nome = nome_visivel(&tag.texto());
        if nome != tipo {
            continue;
        }
        let onclick = tag.atributo("onclick").map(str::to_owned);
        let href = tag
            .atributo("href")
            .map(str::to_owned)
            .ok_or(ErroCriar::LinkDoTipoNaoNavegavel)?;
        if link_de_tipo_externo(&href, onclick.as_deref()) {
            continue;
        }
        links.push((href, onclick));
    }
    let (href, onclick) = match links.as_slice() {
        [] => return Err(ErroCriar::TipoAusente),
        [link] => link,
        _ => return Err(ErroCriar::TipoAmbiguo),
    };
    if href.starts_with("controlador.php?") {
        let link = link_do_tipo::conferir_link_do_tipo(instalacao, href, id_procedimento)?;
        return Ok(EscolhaDoDocumento::Link(link));
    }
    if href != "#" && href != "javascript:void(0);" {
        return Err(ErroCriar::LinkDoTipoNaoNavegavel);
    }
    let id = onclick
        .as_deref()
        .and_then(id_da_escolha_literal)
        .ok_or(ErroCriar::LinkDoTipoNaoNavegavel)?;
    script_prova_escolha(html).map_err(ErroCriar::NavegacaoNaoComprovadaPorScript)?;
    Ok(EscolhaDoDocumento::Formulario(id))
}

/// O redirecionamento do POST de escolha só pode abrir o cadastro assinado
/// do mesmo processo. Suporta Location relativa ou absoluta da instalação.
pub(crate) fn cadastro_redirecionado(
    instalacao: &Instalacao,
    location: &str,
    id_procedimento: &str,
) -> Result<String, ErroCriar> {
    let base = format!("{}/", instalacao.url_base());
    let relativo = location.strip_prefix(&base).unwrap_or(location);
    conferir_endereco_relativo(
        instalacao,
        relativo,
        AcaoDoSei::DocumentoGerar,
        id_procedimento,
    )
    .map_err(ErroCriar::EnderecoDoTipoNaoReconhecido)
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cadastro_obrigatorio;
#[cfg(test)]
mod tests_scripts_inertes;
#[cfg(test)]
mod tests_telas_reais;

#[cfg(test)]
#[path = "documento_criar/tests_anties.rs"]
mod tests_anties;

#[cfg(test)]
#[path = "documento_criar/tests_barra_tramitacao.rs"]
mod tests_barra_tramitacao;
