//! A consulta (`?...`) de uma URL do `controlador.php` lida uma vez só (#751).
//! Cada par precisa de `=`, a chave e o valor passam pelas mesmas regras do
//! transporte (`conferir_chave_de_consulta`, `conferir_valor_de_consulta`) e
//! a leitura de uma chave exige exatamente um valor.
use crate::acao_do_sei::AcaoDoSei;
use crate::transporte::{FalhaTransporte, conferir_chave_de_consulta, conferir_valor_de_consulta};

/// Os pares `chave=valor` da consulta, na ordem publicada, emprestados da URL.
pub(crate) struct ConsultaDoControlador<'a> {
    pares: Vec<(&'a str, &'a str)>,
}

/// A consulta não se deixa ler em pares.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroConsultaDoControlador {
    /// Um trecho entre `&` não tem `=`.
    ParSemIgual,
    /// A chave ou o valor fora da forma que o transporte aceita enviar.
    RecusadaPeloTransporte(#[from] FalhaTransporte),
}

/// A chave pedida não tem exatamente um valor aceito.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroChaveDaConsulta {
    Ausente {
        chave: &'static str,
    },
    Repetida {
        chave: &'static str,
    },
    /// `acao=` ou `acao_origem=` com valor fora de `AcaoDoSei`.
    AcaoDesconhecida,
}

impl<'a> ConsultaDoControlador<'a> {
    /// A consulta de uma URL, que pode não ter `?`: sem ela, não há pares.
    pub(crate) fn da_url(consulta: Option<&'a str>) -> Result<Self, ErroConsultaDoControlador> {
        consulta.map_or_else(|| Ok(Self { pares: Vec::new() }), Self::ler)
    }

    /// `consulta` é o texto depois do `?`, sem ele; vazia, não tem pares.
    pub(crate) fn ler(consulta: &'a str) -> Result<Self, ErroConsultaDoControlador> {
        if consulta.is_empty() {
            return Ok(Self { pares: Vec::new() });
        }
        let pares = consulta
            .split('&')
            .map(|par| {
                let (chave, valor) = par
                    .split_once('=')
                    .ok_or(ErroConsultaDoControlador::ParSemIgual)?;
                conferir_chave_de_consulta(chave)?;
                conferir_valor_de_consulta(valor)?;
                Ok((chave, valor))
            })
            .collect::<Result<_, ErroConsultaDoControlador>>()?;
        Ok(Self { pares })
    }

    /// O único valor de `chave`.
    pub(crate) fn unico(&self, chave: &'static str) -> Result<&'a str, ErroChaveDaConsulta> {
        let mut valores = self.valores(chave);
        let valor = valores
            .next()
            .ok_or(ErroChaveDaConsulta::Ausente { chave })?;
        match valores.next() {
            Some(_) => Err(ErroChaveDaConsulta::Repetida { chave }),
            None => Ok(valor),
        }
    }

    /// A única `acao=`, lida no conjunto fechado do SEI.
    pub(crate) fn acao(&self) -> Result<AcaoDoSei, ErroChaveDaConsulta> {
        self.acao_em("acao")
    }

    /// A única `acao_origem=`, lida no conjunto fechado do SEI.
    pub(crate) fn acao_de_origem(&self) -> Result<AcaoDoSei, ErroChaveDaConsulta> {
        self.acao_em("acao_origem")
    }

    fn acao_em(&self, chave: &'static str) -> Result<AcaoDoSei, ErroChaveDaConsulta> {
        AcaoDoSei::ler(self.unico(chave)?).ok_or(ErroChaveDaConsulta::AcaoDesconhecida)
    }

    /// Todos os valores de `chave`, na ordem publicada.
    pub(crate) fn valores(&self, chave: &str) -> impl Iterator<Item = &'a str> {
        self.pares
            .iter()
            .filter(move |(nome, _)| *nome == chave)
            .map(|(_, valor)| *valor)
    }
}

/// Por que a URL não traz um id decimal único na chave pedida.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroIdDaUrl {
    /// A URL não tem `?`.
    SemConsulta,
    /// A URL relativa não é do `controlador.php`.
    ForaDoControlador,
    /// Um trecho entre `&` não tem `=`.
    ParSemIgual,
    /// A chave ou o valor fora da forma que o transporte aceita enviar.
    RecusadaPeloTransporte(#[source] FalhaTransporte),
    ParametroAusente(&'static str),
    ParametroRepetido(&'static str),
    /// O valor não é decimal; ids do SEI são números, então o valor vai junto.
    IdNaoDecimal {
        chave: &'static str,
        valor: String,
    },
}

/// O id decimal único de `chave` numa URL relativa do `controlador.php?`.
pub(crate) fn id_decimal_do_controlador<'a>(
    url: &'a str,
    chave: &'static str,
) -> Result<&'a str, ErroIdDaUrl> {
    if !url.starts_with("controlador.php?") {
        return Err(ErroIdDaUrl::ForaDoControlador);
    }
    id_decimal(url, chave)
}

/// O id decimal único de `chave` na consulta de `url` (texto depois do `?`).
pub(crate) fn id_decimal<'a>(url: &'a str, chave: &'static str) -> Result<&'a str, ErroIdDaUrl> {
    let (_, texto) = url.split_once('?').ok_or(ErroIdDaUrl::SemConsulta)?;
    let consulta = ConsultaDoControlador::ler(texto).map_err(|erro| match erro {
        ErroConsultaDoControlador::ParSemIgual => ErroIdDaUrl::ParSemIgual,
        ErroConsultaDoControlador::RecusadaPeloTransporte(falha) => {
            ErroIdDaUrl::RecusadaPeloTransporte(falha)
        }
    })?;
    let mut valores = consulta.valores(chave);
    let valor = valores.next().ok_or(ErroIdDaUrl::ParametroAusente(chave))?;
    if valores.next().is_some() {
        return Err(ErroIdDaUrl::ParametroRepetido(chave));
    }
    if valor.is_empty() || !valor.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroIdDaUrl::IdNaoDecimal {
            chave,
            valor: valor.to_owned(),
        });
    }
    Ok(valor)
}

/// Seleção de candidato, tolerante de propósito: a consulta de `href` (o
/// texto depois do `?`) tem algum par `acao=` com o valor de `acao`? Não
/// confere o resto da consulta; quem seleciona por aqui confere o candidato
/// escolhido depois, de forma estrita (`conferir_endereco_relativo` ou
/// `ConsultaDoControlador::ler`), para que um link malformado seja recusado
/// e não descartado como ausente.
pub(crate) fn menciona_acao(href: &str, acao: AcaoDoSei) -> bool {
    href.split_once('?').is_some_and(|(_, consulta)| {
        consulta
            .split('&')
            .any(|par| par.strip_prefix("acao=") == Some(acao.como_na_url()))
    })
}

#[cfg(test)]
#[path = "tests_consulta_do_controlador.rs"]
mod tests;
