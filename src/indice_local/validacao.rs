//! Conferência da entrada antes de gravar, e a URL assinada redigida.
//! Sem expressões regulares: só comparação de bytes.
use super::ChaveDoLink;
use crate::instalacao::Instalacao;
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};
use std::fmt;

/// Entrada recusada antes de tocar no banco. Sem dados: nunca carrega a URL,
/// o hash, a geração ou o NUP. O chamador pode ignorá-la e seguir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RegistroRecusado {
    NupInvalido,
    NumeroSeiInvalido,
    IdProcedimentoInvalido,
    GeracaoInvalida,
    UnidadeInvalida,
    ObjetoInvalido,
    LinkNaoHttps,
    LinkComCaractereProibido,
    LinkForaDaInstalacao,
    LinkComConsultaNaoCanonica,
    LinkSemAcaoUnicaIgual,
    LinkSemInfraHashUnico,
    LinkComAcaoAjax,
    LinkDeOutraUnidade,
}

/// URL assinada como a tela publicou. `Debug` não a mostra; não há `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct UrlAssinada(String);

impl UrlAssinada {
    pub(super) fn guardada(url: String) -> Self {
        Self(url)
    }

    pub(crate) fn url(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for UrlAssinada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UrlAssinada(<redigida>)")
    }
}

/// Maior URL guardada; as observadas têm poucas centenas de bytes.
const LIMITE_DA_URL: usize = 4096;

fn texto_sem_controle(valor: &str, maximo: usize) -> bool {
    valor.len() <= maximo && !valor.chars().any(char::is_control)
}

fn imprimivel(valor: &str, maximo: usize) -> bool {
    !valor.is_empty() && valor.len() <= maximo && valor.bytes().all(|b| (33..=126).contains(&b))
}

/// NUP opaco: não vazio, sem caracteres de controle. Nunca normalizado.
pub(super) fn conferir_nup(nup: &str) -> Result<(), RegistroRecusado> {
    if nup.is_empty() || !texto_sem_controle(nup, 128) {
        return Err(RegistroRecusado::NupInvalido);
    }
    Ok(())
}

/// Número SEI opaco (zeros à esquerda inclusos): não vazio, sem controle.
/// Nunca vai para URL; é só a chave da busca.
pub(super) fn conferir_numero_sei(numero: &str) -> Result<(), RegistroRecusado> {
    if numero.is_empty() || !texto_sem_controle(numero, 64) {
        return Err(RegistroRecusado::NumeroSeiInvalido);
    }
    Ok(())
}

/// O id vai para uma URL montada pelo chamador: só dígitos ASCII.
pub(super) fn conferir_id_procedimento(id: &str) -> Result<(), RegistroRecusado> {
    if id.is_empty() || id.len() > 20 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(RegistroRecusado::IdProcedimentoInvalido);
    }
    Ok(())
}

pub(super) fn conferir_geracao(geracao: &str) -> Result<(), RegistroRecusado> {
    if !imprimivel(geracao, 128) {
        return Err(RegistroRecusado::GeracaoInvalida);
    }
    Ok(())
}

pub(super) fn conferir_chave(chave: ChaveDoLink<'_>) -> Result<(), RegistroRecusado> {
    conferir_geracao(chave.geracao_sessao)?;
    if !imprimivel(chave.unidade, 64) {
        return Err(RegistroRecusado::UnidadeInvalida);
    }
    if !texto_sem_controle(chave.alvo, 128) {
        return Err(RegistroRecusado::ObjetoInvalido);
    }
    Ok(())
}

fn hash_valido(valor: &str) -> bool {
    valor.len() == 64 && valor.bytes().all(|b| b.is_ascii_hexdigit())
}

/// URL absoluta de `controlador.php` da instalação, com exatamente um
/// `acao` igual ao da chave e um `infra_hash` de 64 hex, sem `acao_ajax`, e
/// `infra_unidade_atual` (se houver) igual à unidade da chave.
pub(super) fn conferir_url(
    instalacao: &Instalacao,
    chave: ChaveDoLink<'_>,
    url: &str,
) -> Result<(), RegistroRecusado> {
    let restante = url
        .strip_prefix("https://")
        .ok_or(RegistroRecusado::LinkNaoHttps)?;
    if url.len() > LIMITE_DA_URL
        || !url.bytes().all(|b| (33..=126).contains(&b))
        || url.contains(['#', '\\'])
    {
        return Err(RegistroRecusado::LinkComCaractereProibido);
    }
    let (endereco, consulta) = restante
        .split_once('?')
        .ok_or(RegistroRecusado::LinkForaDaInstalacao)?;
    let (host, caminho) = endereco
        .split_once('/')
        .ok_or(RegistroRecusado::LinkForaDaInstalacao)?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(RegistroRecusado::LinkForaDaInstalacao);
    }
    let (mut acoes, mut hashes) = (0, 0);
    for par in consulta.split('&') {
        let (nome, valor) = par
            .split_once('=')
            .ok_or(RegistroRecusado::LinkComConsultaNaoCanonica)?;
        if conferir_chave_de_consulta(nome).is_err() || conferir_valor_de_consulta(valor).is_err() {
            return Err(RegistroRecusado::LinkComConsultaNaoCanonica);
        }
        match nome {
            "acao" if valor == chave.acao.como_na_url() => acoes += 1,
            "acao" => return Err(RegistroRecusado::LinkSemAcaoUnicaIgual),
            "infra_hash" if hash_valido(valor) => hashes += 1,
            "infra_hash" => return Err(RegistroRecusado::LinkSemInfraHashUnico),
            "acao_ajax" => return Err(RegistroRecusado::LinkComAcaoAjax),
            "infra_unidade_atual" if valor != chave.unidade => {
                return Err(RegistroRecusado::LinkDeOutraUnidade);
            }
            _ => {}
        }
    }
    if acoes != 1 {
        return Err(RegistroRecusado::LinkSemAcaoUnicaIgual);
    }
    if hashes != 1 {
        return Err(RegistroRecusado::LinkSemInfraHashUnico);
    }
    Ok(())
}
