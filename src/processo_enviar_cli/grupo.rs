//! Prévia de grupo (#875): reutiliza a consulta de grupo e o envio plural.
//! A confirmação continua indisponível; a união só é publicada após conferir
//! cada identidade lexical do grupo contra os destinos resolvidos.
use super::{resultado_envio, sigla_de_destino};
use crate::grupo_envio_cli::{Grupo, Membro, obter_via};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::Contador;
use crate::processo_concluir_cli::com_sessao_referencias;
use crate::processo_enviar::{Destino, PedidoDoEnvio, enviar_processo};
use crate::visao_do_controle::EnvioHttps;
use crate::Cli;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroGrupoDaPrevia {
    IdDeGrupoInvalido,
    GrupoSemMembros,
    MembroSemSigla { id: String },
    IdentidadeDoMembroDivergente { id: String },
}

pub(crate) fn id_de_grupo(id: &str) -> Result<String, ErroGrupoDaPrevia> {
    if id.is_empty() || id.trim() != id || id.chars().any(char::is_control) {
        return Err(ErroGrupoDaPrevia::IdDeGrupoInvalido);
    }
    Ok(id.to_owned())
}

fn sigla(membro: &Membro) -> Result<String, ErroGrupoDaPrevia> {
    let (sigla, _) = membro.nome.split_once(" - ").ok_or_else(|| {
        ErroGrupoDaPrevia::MembroSemSigla {
            id: membro.id.clone(),
        }
    })?;
    sigla_de_destino(sigla).map_err(|_| ErroGrupoDaPrevia::MembroSemSigla {
        id: membro.id.clone(),
    })
}

fn siglas_do_grupo(grupo: &Grupo) -> Result<Vec<String>, ErroGrupoDaPrevia> {
    if grupo.unidades.is_empty() {
        return Err(ErroGrupoDaPrevia::GrupoSemMembros);
    }
    grupo.unidades.iter().map(sigla).collect()
}

fn conferir_membros(grupo: &Grupo, destinos: &[Destino]) -> Result<(), ErroGrupoDaPrevia> {
    for membro in &grupo.unidades {
        let sigla = sigla(membro)?;
        if !destinos
            .iter()
            .any(|d| d.id_unidade == membro.id && d.sigla.eq_ignore_ascii_case(&sigla))
        {
            return Err(ErroGrupoDaPrevia::IdentidadeDoMembroDivergente {
                id: membro.id.clone(),
            });
        }
    }
    Ok(())
}

pub(super) fn prever(
    cli: &Cli,
    nup: &NumeroProcesso,
    id: &str,
    para: &[String],
    manter_aberto: bool,
    remover_anotacoes: bool,
) -> anyhow::Result<()> {
    com_sessao_referencias(cli, |sessao, indice, referencias| {
        let mut https = EnvioHttps;
        let mut rede = Contador::novo(&mut https);
        let (unidade, grupo) =
            obter_via(referencias, cli.unidade.as_deref(), id, &mut rede)?;
        // Membros primeiro: uma sigla explícita não pode esconder a identidade
        // de um membro quando o fluxo plural deduplica pelo ID.
        let mut siglas = siglas_do_grupo(&grupo)?;
        siglas.extend_from_slice(para);
        let (primeiro, outros) = siglas
            .split_first()
            .ok_or(ErroGrupoDaPrevia::GrupoSemMembros)?;
        let outros: Vec<_> = outros.iter().map(String::as_str).collect();
        let pedido = PedidoDoEnvio {
            nup,
            unidade: Some(&unidade),
            para: primeiro,
            outros: &outros,
            manter_aberto,
            remover_anotacoes,
            confirmar: false,
        };
        let mut feito = enviar_processo(sessao, &pedido, indice, &mut rede)?;
        conferir_membros(&grupo, &feito.destinos)?;
        feito.requisicoes = rede.feitas();
        if cli.json {
            let mut valor = serde_json::to_value(resultado_envio(&feito))?;
            valor["grupo_id"] = serde_json::Value::String(grupo.id);
            crate::indice_local::anexar_avisos(&mut valor, indice);
            println!("{}", serde_json::to_string_pretty(&valor)?);
        } else {
            println!("{}", super::humano::humano_com_grupo(&feito, Some(&grupo.id)));
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grupo(membros: &[(&str, &str)]) -> Grupo {
        Grupo {
            id: "004".into(),
            nome: "Grupo".into(),
            descricao: String::new(),
            unidades: membros
                .iter()
                .map(|(id, nome)| Membro {
                    id: (*id).into(),
                    nome: (*nome).into(),
                })
                .collect(),
        }
    }

    fn destino(id: &str, sigla: &str) -> Destino {
        Destino {
            id_unidade: id.into(),
            sigla: sigla.into(),
            descricao: format!("{sigla} - Unidade"),
        }
    }

    #[test]
    fn grupo_preserva_identidade_lexical_e_recusa_homonimo() {
        let g = grupo(&[("001", "X - Unidade"), ("002", "Y - Unidade")]);
        assert_eq!(id_de_grupo("004").unwrap(), "004");
        assert_eq!(siglas_do_grupo(&g).unwrap(), ["X", "Y"]);
        assert!(conferir_membros(&g, &[destino("001", "x"), destino("002", "Y")]).is_ok());
        for destinos in [
            vec![destino("1", "X"), destino("002", "Y")],
            vec![destino("002", "X"), destino("001", "Y")],
            vec![destino("001", "X")],
        ] {
            assert!(conferir_membros(&g, &destinos).is_err());
        }
    }

    #[test]
    fn grupo_vazio_ou_sem_sigla_nao_define_destinos() {
        assert!(siglas_do_grupo(&grupo(&[])).is_err());
        assert!(siglas_do_grupo(&grupo(&[("001", "Homônimo")])).is_err());
        for id in ["", " 004", "004 ", "0\t04"] {
            assert!(id_de_grupo(id).is_err());
        }
    }
}
