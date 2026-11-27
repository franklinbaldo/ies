//! Resolução de um número pelas listas de blocos publicadas.
use super::{
    EnvioDoControle, ErroBloco, FalhaLeitura, Leitura, Lido, ReferenciasDaSessao, TipoBloco, bloco,
    bloco_detalhe, controle_reconhecido, ler_tela_via, transporte,
};
use crate::acao_do_sei::AcaoDoSei;
use crate::bloco_estado::EstadoDoBloco;

/// Resolve um número pelas listas tipadas efetivamente publicadas, sem
/// escolher assinatura por padrão. O número do bloco é uma sequência única
/// da instalação (`UnB`: 2503 assinatura, 2502 interno), então a página de
/// outro tipo não lida não esconde colisão.
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
    let mut identidade: Option<(TipoBloco, String, EstadoDoBloco)> = None;
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
            identidade = Some((tipo, url, item.estado));
        }
    }
    let Some((tipo, url, estado)) = identidade else {
        // Respeita o contrato antigo: o número não foi observado na página.
        return Err(ErroBloco::BlocoNaoEncontradoNaPagina.into());
    };
    requisicoes += 1;
    let resposta = envio.get(&url, cookie)?;
    let html = transporte::decodificar_resposta(resposta)?;
    let detalhe =
        bloco_detalhe::ler_detalhe(&html, numero, tipo, estado).map_err(FalhaLeitura::Detalhe)?;
    Ok(Lido {
        unidade,
        requisicoes,
        leitura: Leitura::Detalhe(detalhe),
        escopo_criacao: false,
    })
}
