//! Paginação da lista de blocos internos por formulário observado, sem JS.
use super::FalhaLeitura;
use crate::acao_do_sei::AcaoDoSei;
use crate::bloco::{self, ErroBloco, ListaComLinks, TipoBloco};
use crate::formulario_web::{
    CodificacaoDaTela, ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, derivar_submissao,
    enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::EnvioDoControle;

/// Navegação observada na cassette GO bloco-interno-ciclo, passo 12 GET
/// seguido de passo 13 POST: índice zero-based 1 -> faixa 51 a 100.
/// O corpo do POST original nao foi preservado, logo a derivacao segue
/// estritamente os controles que a resposta HTML publicou.
pub(super) const PAGINAR_BLOCOS: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::BlocoInternoListar,
    formulario: "frmBlocoLista",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnInfraPaginaAtual", "hdnInfraTipoPagina"],
    campos_ocultos_reescritos: &["hdnInfraPaginaAtual"],
    campos_usuario: &["selInfraPaginacaoSuperior", "selInfraPaginacaoInferior"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

/// Prova que o formulário serviu exatamente a opção de destino. Sem executar JS.
pub(super) fn conferir_indice_da_pagina(
    html: &crate::tela_html::HtmlServido,
    indice: &str,
) -> Result<(), FalhaLeitura> {
    let ilegivel = || FalhaLeitura::HtmlDeBlocosInvalido;
    let dom = crate::tela_html::arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, "frmBlocoLista")?;
    let mut achados = 0;
    for no in form.elementos("select") {
        if no.atributo("name") != Some("selInfraPaginacaoSuperior") {
            continue;
        }
        achados += 1;
        let opcoes = no.elementos("option");
        let valores: Vec<_> = opcoes.map(|o| o.atributo("value")).collect();
        if valores.iter().filter(|v| **v == Some(indice)).count() != 1 {
            return Err(ilegivel());
        }
    }
    if achados != 1 {
        return Err(ilegivel());
    }
    Ok(())
}

/// Abre apenas a segunda página a partir dos controles publicados na primeira.
pub(super) fn ler_pagina_dois<V: EnvioDoControle>(
    lista: &ListaComLinks,
    html: &crate::tela_html::HtmlServido,
    url_lista: &str,
    codificacao: Option<CodificacaoDaTela>,
    instalacao: &Instalacao,
    sessao: &ReferenciasDaSessao,
    envio: &mut V,
) -> Result<ListaComLinks, FalhaLeitura> {
    // GO: a segunda página é obtida por um único POST ao formulário da
    // lista, sem navegar por páginas intermediárias ou restaurar a tela.
    let primeira = lista
        .lista
        .faixa_observada
        .as_ref()
        .ok_or(FalhaLeitura::Bloco(ErroBloco::FaixaDeBlocosIncoerente))?;
    let total = lista.lista.total_declarado;
    let fim_primeira = primeira.fim;
    if primeira.inicio != 1 || primeira.fim == total {
        return Err(FalhaLeitura::Bloco(ErroBloco::FaixaDeBlocosIncoerente));
    }
    conferir_indice_da_pagina(html, "1")?;
    let codificacao = codificacao.ok_or(FalhaLeitura::Bloco(
        ErroBloco::PaginaDeBlocosNaoImplementada,
    ))?;
    let indice = "1";
    let tela = TelaServida {
        url: url_lista,
        html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &PAGINAR_BLOCOS,
        instalacao,
        &tela,
        None,
        &[
            ("hdnInfraPaginaAtual", indice),
            ("selInfraPaginacaoSuperior", indice),
            ("selInfraPaginacaoInferior", indice),
        ],
    )?;
    let resposta = enviar_submissao_via(submissao, sessao.cookie(), |post, cookie| {
        envio.post(post, cookie)
    })?;
    let RespostaDoEnvio::Html(html) = resposta else {
        return Err(FalhaLeitura::Bloco(ErroBloco::FaixaDeBlocosIncoerente));
    };
    let lista_segunda = bloco::ler_lista(&html, TipoBloco::Interno)?;
    let faixa = lista_segunda
        .lista
        .faixa_observada
        .as_ref()
        .ok_or(FalhaLeitura::Bloco(ErroBloco::FaixaDeBlocosIncoerente))?;
    if lista_segunda.lista.total_declarado != total
        || faixa.inicio != fim_primeira + 1
        || faixa.fim > total
    {
        return Err(FalhaLeitura::Bloco(ErroBloco::FaixaDeBlocosIncoerente));
    }
    Ok(lista_segunda)
}
