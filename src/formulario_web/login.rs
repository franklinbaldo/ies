//! Derivação do POST de login: o mesmo primitivo, com a tela e o `action`
//! no **host** da instalação (o SIP `sip/login.php` e o módulo de Goiás
//! ficam fora do caminho-base do `controlador.php`). Não confere `acao=`
//! nem `infra_hash`, que o login não tem; o resto das travas é o mesmo.
use super::{ErroSubmissaoFormulario, OperacaoDeFormulario, SubmissaoDerivada, TelaServida};
use crate::instalacao::Instalacao;

/// Deriva o POST do formulário de login servido, sem rede.
pub(crate) fn derivar_submissao_de_login(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    let _ = (operacao, instalacao, tela, campos);
    todo!("CONTRACT: derivação do login")
}
