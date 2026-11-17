//! Derivação do POST de login: o mesmo primitivo, com a tela e o `action`
//! no **host** da instalação (o SIP `sip/login.php` e o módulo de Goiás
//! ficam fora do caminho-base do `controlador.php`). Não confere `acao=`
//! nem `infra_hash`, que o login não tem, nem a unidade, que a tela de
//! login não publica; o resto das travas é o mesmo.
use super::leitura::{
    UrlHttps, conferir_metodo_e_codificacao, ler_controles, localizar_formulario,
    resolver_action_no_host,
};
use super::{
    EfeitoAvaliado, ErroSubmissaoFormulario, OperacaoDeFormulario, SubmissaoDerivada, TelaServida,
    codificar_corpo, ordenar_campos,
};
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;

/// Deriva o POST do formulário de login servido, sem rede.
pub(crate) fn derivar_submissao_de_login(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    if operacao.efeito == EfeitoAvaliado::Desconhecido {
        return Err(ErroSubmissaoFormulario::EfeitoDaOperacaoNaoAvaliado);
    }
    if tela.html.len() > crate::MAX_HTML_BYTES {
        return Err(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::RespostaExcessiva,
        ));
    }
    let servida = UrlHttps::interpretar(tela.url).ok_or(ErroSubmissaoFormulario::Transporte(
        FalhaTransporte::UrlInvalida,
    ))?;
    if !servida.host.eq_ignore_ascii_case(instalacao.host()) {
        return Err(ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::OrigemNaoAutorizada,
        ));
    }
    let dom = crate::tela_html::arvore(tela.html);
    let form = localizar_formulario(&dom, operacao.formulario)?;
    conferir_metodo_e_codificacao(form)?;
    let url = resolver_action_no_host(form, &servida, instalacao)?;
    let controles = ler_controles(form, operacao)?;
    let ordenados = ordenar_campos(&controles, operacao, campos)?;
    let corpo = codificar_corpo(&ordenados, tela.codificacao)?;
    Ok(SubmissaoDerivada {
        url,
        host: servida.host.to_ascii_lowercase(),
        caminho: servida.caminho.to_owned(),
        acao: crate::acao_do_sei::AcaoDaSubmissao::Controlador(operacao.acao),
        referer: tela.url.to_owned(),
        nomes: ordenados.iter().map(|(n, _)| (*n).to_owned()).collect(),
        corpo: corpo.into_bytes(),
        efeito: operacao.efeito,
        resultado: operacao.resultado,
    })
}
