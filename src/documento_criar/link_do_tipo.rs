//! Links de tipo publicados na lista de documentos (#246).
//! O oráculo SEI 4.0.9 repete `id_procedimento` com o mesmo valor.
//! Conserva a URL assinada original para o GET; só a cópia usada para
//! validação perde a segunda ocorrência idêntica.
use super::ErroListaDeTiposDocumento;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::endereco::conferir_endereco_com_identidade_repetida;
use crate::instalacao::Instalacao;

pub(super) fn conferir_link_do_tipo(
    instalacao: &Instalacao,
    href: &str,
    id_procedimento: &str,
) -> Result<String, ErroListaDeTiposDocumento> {
    conferir_endereco_com_identidade_repetida(
        instalacao,
        href,
        AcaoDoSei::DocumentoGerar,
        id_procedimento,
    )
    .map_err(ErroListaDeTiposDocumento::EnderecoDoTipoNaoReconhecido)
}
