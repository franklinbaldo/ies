//! Consultar uma versão observada em histórico Web hash-verificado, sem rede.
use crate::cassette;
use crate::documento::{NumeroDocumento, NumeroDocumentoInvalido};
use crate::documento_versoes::{self, ErroVersao, VersaoDocumento};
use serde::Serialize;
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroConsultaVersoes {
    NumeroDocumento(NumeroDocumentoInvalido),
    NumeroVersao(ErroVersao),
    IdInternoInvalido,
    FonteNaoVerificada,
}
impl ErroConsultaVersoes {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::NumeroDocumento(erro) => erro.subcodigo(),
            Self::NumeroVersao(erro) => erro.subcodigo(),
            Self::IdInternoInvalido => "id_interno_documento_invalido",
            Self::FonteNaoVerificada => "historico_versoes_nao_verificado",
        }
    }
    fn codigo(self) -> (&'static str, u8) {
        match self {
            Self::NumeroDocumento(_) | Self::IdInternoInvalido
            | Self::NumeroVersao(ErroVersao::NumeroInvalido) => ("entrada_invalida", 2),
            _ => ("fonte_invalida", 1),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ConsultaVersoes {
    versao_atual: Option<u32>,
    versao_selecionada: Option<u32>,
    total_observado: usize,
}

/// A tabela de versões declara ID interno, NÃO comprova associação ao número SEI.
fn consultar(
    html: &str,
    documento: &NumeroDocumento,
    id_interno: &str,
    solicitada: Option<VersaoDocumento>,
) -> Result<ConsultaVersoes, ErroConsultaVersoes> {
    if id_interno.is_empty() || !id_interno.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroConsultaVersoes::IdInternoInvalido);
    }
    let historico = documento_versoes::interpretar_historico(
        html, documento, id_interno, None
    ).map_err(ErroConsultaVersoes::NumeroVersao)?;
    let selecionada = solicitada
        .map(|versao| historico.selecionar(versao))
        .transpose()
        .map_err(ErroConsultaVersoes::NumeroVersao)?;
    Ok(ConsultaVersoes {
        versao_atual: historico.atual().map(VersaoDocumento::numero),
        versao_selecionada: selecionada.map(|entrada| entrada.versao.numero()),
        total_observado: historico.len(),
    })
}

pub(super) fn mostrar(
    card: &Path,
    arquivo: &Path,
    passos: &[usize],
    numero: Option<&str>,
    id_interno: Option<&str>,
    versao: Option<u32>,
    json_mode: bool,
) -> ExitCode {
    let resultado = (|| {
        let numero = NumeroDocumento::novo(numero.unwrap_or(""))
            .map_err(ErroConsultaVersoes::NumeroDocumento)?;
        let id_interno = id_interno.ok_or(ErroConsultaVersoes::IdInternoInvalido)?;
        if id_interno.is_empty() || !id_interno.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroConsultaVersoes::IdInternoInvalido);
        }
        let versao = versao
            .map(VersaoDocumento::nova)
            .transpose()
            .map_err(ErroConsultaVersoes::NumeroVersao)?;
        let (html, instalacao) = cassette::read_cassette_document_versions(card, arquivo, passos)
            .map_err(|_| ErroConsultaVersoes::FonteNaoVerificada)?;
        let consulta = consultar(&html, &numero, id_interno, versao)?;
        Ok((numero, id_interno, consulta, instalacao))
    })();
    match resultado {
        Ok((documento, id_interno, consulta, instalacao)) => {
            if json_mode {
                println!("{}", json!({
                    "numero_documento_informado": documento.as_str(),
                    "numero_documento_confirmado": false,
                    "id_interno_documento": id_interno,
                    "instalacao": instalacao,
                    "escopo": "historico_versoes_cassette_offline",
                    "versao_atual": consulta.versao_atual,
                    "versao_selecionada": consulta.versao_selecionada,
                    "total_observado": consulta.total_observado
                }));
            } else {
                println!(
                    "IES: {} versão(ões) observadas para ID interno {id_interno}; versão atual observada: {:?}. O vínculo ao número SEI informado não foi comprovado.",
                    consulta.total_observado, consulta.versao_atual
                );
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            let (codigo, saida) = erro.codigo();
            if json_mode {
                println!("{}", json!({"codigo":codigo,"subcodigo":erro.subcodigo(),"retentavel":false}));
            } else {
                eprintln!("IES: histórico de versões offline recusado [{}]", erro.subcodigo());
            }
            ExitCode::from(saida)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pagina(numeros: &[u32], id: &str) -> String {
        use std::fmt::Write as _;
        let mut rows = String::new();
        for n in numeros {
            write!(rows, "<tr><td></td><td>{n}</td><td>Ana</td><td>PPI</td><td>31/10/2026</td><td><a href='controlador.php?acao=documento_visualizar&amp;id_documento={id}&amp;versao={n}&amp;infra_hash={}'>Ver</a></td></tr>", "a".repeat(64)).unwrap();
        }
        format!("<table class='infraTable'><caption>Lista de Versões ({} registros):</caption><tr><th></th><th>Versão</th><th>Usuário</th><th>Unidade</th><th>Última Modificação</th><th>Ações</th></tr>{rows}</table>", numeros.len())
    }

    #[test]
    fn atual_observada_nao_infere_versoes_ausentes() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        let resultado = consultar(&pagina(&[1,4], "42"), &doc, "42", None).unwrap();
        assert_eq!(resultado.versao_atual, Some(4));
        assert_eq!(resultado.versao_selecionada, None);
        assert_eq!(resultado.total_observado, 2);
    }

    #[test]
    fn selecao_pontual_nao_retorna_link_assinado() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        let resultado = consultar(&pagina(&[1,4], "42"), &doc, "42", Some(VersaoDocumento::nova(4).unwrap())).unwrap();
        assert_eq!(resultado.versao_selecionada, Some(4));
        assert!(!serde_json::to_string(&resultado).unwrap().contains("infra_hash"));
    }

    #[test]
    fn ausencia_na_tabela_nao_fabrica_versao_historica() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        assert_eq!(
            consultar(&pagina(&[1,4], "42"), &doc, "42", Some(VersaoDocumento::nova(2).unwrap())),
            Err(ErroConsultaVersoes::NumeroVersao(ErroVersao::VersaoNaoObservada))
        );
    }

    #[test]
    fn id_interno_diferente_eh_recusado_por_variante_especifica() {
        let doc = NumeroDocumento::novo("0007").unwrap();
        assert_eq!(
            consultar(&pagina(&[1], "43"), &doc, "42", None),
            Err(ErroConsultaVersoes::NumeroVersao(ErroVersao::DocumentoDivergente))
        );
    }
}
