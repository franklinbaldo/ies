//! Modal de revalidacao de processo, exclusivamente em ambiente sintetico.
//! O formulario e uma hipotese BDD-633-01/03, nao uma captura de GO.
//! A senha recebida so e verificada como presente; nunca fica no SQLite.
use crate::{Estado, Resposta, documento_versoes};
use iesei::instalacao::Instalacao;
use iesei::validacao_acesso::{self, Validacao};
use rusqlite::Connection;

/// Nenhuma rota de validacao existe fora do controlador local.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    let id = match documento_versoes::parametro(alvo, "id_procedimento") {
        Ok(Some(id)) if id.bytes().all(|b| b.is_ascii_digit()) => id,
        _ => return Resposta::nova(Estado::RequisicaoInvalida),
    };
    let situacao = match validacao_acesso::ler(banco, instalacao, cenario, id) {
        Ok(Some(situacao)) => situacao,
        Ok(None) => return Resposta::nova(Estado::NaoEncontrado),
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    if metodo == "GET" {
        if situacao == Validacao::Realizada {
            return Resposta::nova(Estado::NaoEncontrado);
        }
        // Nomes e acao deste formulario sao SUPOSICOES testaveis. Nao se
        // transporta este HTML como fato observado nem se gera URL real.
        return Resposta::html(format!(
            "<html><body><form id='frmValidarAcesso' method='post' action='controlador.php?acao=usuario_validar_acesso&amp;id_procedimento={id}&amp;infra_hash={}'>\
             <input type='password' name='txtSenha' /></form></body></html>",
            crate::protocolo_sintetico::INFRA_HASH,
        ));
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    // Um campo de senha nao vazio, sem outros campos ou ambiguidade.
    // O valor nao e guardado, comparado com senha real ou exibido.
    let campos: Vec<_> = corpo.split('&').collect();
    if campos.len() != 1
        || campos[0]
            .strip_prefix("txtSenha=")
            .is_none_or(str::is_empty)
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    if situacao == Validacao::Recusada {
        return Resposta::nova(Estado::AcessoNegado);
    }
    match validacao_acesso::guardar(banco, instalacao, cenario, id, Validacao::Realizada) {
        Ok(()) => Resposta::html(
            "<html><body>Validacao de acesso submetida; confira o processo.</body></html>".into(),
        ),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}
