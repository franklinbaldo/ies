//! Resolução de um número pelas listas de blocos publicadas.
use super::{
    EnvioDoControle, ErroBloco, FalhaLeitura, Leitura, Lido, ReferenciasDaSessao, TipoBloco, bloco,
    bloco_detalhe, controle_reconhecido, ler_tela_via, transporte,
};
use crate::acao_do_sei::AcaoDoSei;

/// Resolve um número pelas listas tipadas efetivamente publicadas, sem
/// escolher assinatura por padrão. Uma página parcial não exclui colisões.
pub(crate) fn ler_bloco_por_numero_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    numero: &str,
    envio: &mut V,
) -> Result<Lido, FalhaLeitura> {
    let mut requisicoes = 1usize;
    let controle = ler_tela_via(
        sessao.url_do_controle(),
        sessao.cookie(),
        sessao.origem(),
        |url, cookie| envio.get(url, cookie),
    )?;
    let unidade = controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade_esperada
        .is_some_and(|v| v.split_whitespace().collect::<Vec<_>>().join(" ") != unidade)
    {
        return Err(FalhaLeitura::UnidadeInesperada);
    }
    let cookie = sessao.cookie().valor()?;
    let mut identidade: Option<(TipoBloco, String, String)> = None;
    let mut lista_parcial_sem_numero = false;
    for tipo in [
        TipoBloco::Assinatura,
        TipoBloco::Interno,
        TipoBloco::Reuniao,
    ] {
        let url_lista =
            match bloco::link_do_menu(&controle.instalacao, &controle.url, &controle.html, tipo) {
                Ok(link) => link,
                Err(ErroBloco::LinkDeBlocosAusente) => continue,
                Err(erro) => return Err(erro.into()),
            };
        requisicoes += 1;
        let resposta = envio.get(&url_lista, cookie)?;
        let html = transporte::decodificar_resposta(resposta)?;
        let lista = bloco::ler_lista(&html, tipo)?;
        if let Some(item) = lista.lista.itens.iter().find(|item| item.numero == numero) {
            if identidade.is_some() {
                return Err(FalhaLeitura::IdentidadeDoBlocoAmbigua);
            }
            let href = lista.href_do_bloco(numero)?;
            let url = bloco::resolver_link(
                &controle.instalacao,
                &url_lista,
                href,
                AcaoDoSei::RelBlocoProtocoloListar,
                Some(numero),
            )?;
            identidade = Some((tipo, url, item.estado.clone()));
        } else if lista.lista.itens.len() != lista.lista.total_declarado {
            lista_parcial_sem_numero = true;
        }
    }
    let Some((tipo, url, estado)) = identidade else {
        // Respeita o contrato antigo: o número não foi observado na página.
        return Err(ErroBloco::BlocoNaoEncontradoNaPagina.into());
    };
    if lista_parcial_sem_numero {
        // Encontrar em A não exclui que a parte não lida de B também o tenha.
        return Err(FalhaLeitura::ListaParcialNaoExcluiColisao);
    }
    requisicoes += 1;
    let resposta = envio.get(&url, cookie)?;
    let html = transporte::decodificar_resposta(resposta)?;
    let detalhe =
        bloco_detalhe::ler_detalhe(&html, numero, tipo, &estado).map_err(FalhaLeitura::Detalhe)?;
    Ok(Lido {
        unidade,
        requisicoes,
        leitura: Leitura::Detalhe(detalhe),
    })
}
