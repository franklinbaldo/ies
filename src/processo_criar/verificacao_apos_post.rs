//! Verificação independente da criação indicada por Location legítimo.
//! ID/NUP na árvore não bastam para confirmar tipo e especificação.
use super::erro_fluxo::ProvaDaCriacao;
use super::{Criacao, ErroCriar, PedidoCriar, get};
use crate::instalacao::Instalacao;
use crate::visao_do_controle::EnvioDoControle;

#[allow(clippy::too_many_arguments)] // cada parâmetro representa um fato da operação.
pub(super) fn verificar<V: EnvioDoControle>(
    envio: &mut V,
    instalacao: &Instalacao,
    url: &str,
    id: &str,
    cookie: &str,
    feito: &Criacao,
    pedido: &PedidoCriar<'_>,
    requisicoes: &mut u32,
) -> Result<String, ErroCriar> {
    use ErroCriar as E;
    let tela_do_processo = get(envio, url, cookie, requisicoes)?;
    let tela_lida = crate::arvore_processo::ler_tela_do_processo(
        instalacao,
        &tela_do_processo.html,
        id,
        Some(&feito.unidade),
    )
    .map_err(E::Tela)?;
    let arvore = get(envio, tela_lida.arvore.as_str(), cookie, requisicoes)?;
    let nup =
        crate::arvore_processo::nup_declarado_na_arvore(&arvore.html).ok_or(E::NaoConfirmado)?;
    crate::arvore_processo::ler_arvore(&arvore.html, &nup, id).map_err(E::Arvore)?;
    let endereco = crate::arvore_processo::ler_link_das_propriedades(instalacao, &arvore.html, id)
        .map_err(E::Propriedades)?
        .ok_or(E::PropriedadesNaoOferecidas)?;
    let pagina = get(envio, endereco.as_str(), cookie, requisicoes)?;
    let propriedades =
        crate::propriedades_processo::ler_propriedades(&pagina.html, &nup, id, endereco.fonte)
            .map_err(E::Propriedades)?;
    if propriedades.tipo.as_deref() != Some(feito.tipo.as_str())
        || propriedades.especificacao != pedido.especificacao
        || propriedades.nivel_de_acesso.como_texto() != pedido.nivel
        || !super::reconciliacao::desfechos::interessados_conferem(
            &feito.interessados,
            &propriedades.interessados,
        )
    {
        return Err(E::ProvaDaCriacao(ProvaDaCriacao::PropriedadesDivergentes));
    }
    Ok(nup)
}
