//! O HTML versionado da minuta, sem extração de texto.
use super::versao::{self, FotoDaVersao};
use super::{
    ConteudoNaArvore, EnvioDoProcesso, ErroDocumentoObter, FimDaLeitura, IndiceLocal, Instalacao,
    NoLido, PedidoDoDocumento, Rede, SessaoDoProcesso, abrir, ler_conteudo_na_arvore,
};

pub(crate) fn obter_foto_atual<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoDocumento<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<FotoDaVersao, ErroDocumentoObter> {
    abrir(sessao, pedido, SoFotoAtual, indice, envio)
}

#[derive(Clone, Copy)]
struct SoFotoAtual;

impl FimDaLeitura for SoFotoAtual {
    type Lido = FotoDaVersao;

    fn concluir<V: EnvioDoProcesso>(
        self,
        instalacao: &Instalacao,
        rede: &mut Rede<'_, V>,
        lido: NoLido<'_>,
        numero: &str,
    ) -> Result<Self::Lido, ErroDocumentoObter> {
        let ConteudoNaArvore::Visualizar(conteudo) =
            ler_conteudo_na_arvore(instalacao, lido.html, &lido.no.id)?
        else {
            return Err(ErroDocumentoObter::ConteudoNaoPublicadoNaArvore {
                requisicoes: rede.requisicoes.total_u8(),
            });
        };
        let conferido = versao::NoConferido {
            instalacao,
            numero,
            html: lido.html,
            id_documento: &lido.no.id,
            id_procedimento: &lido.processo.id,
        };
        versao::ler_foto_atual(rede, &conferido, &conteudo)
    }
}
