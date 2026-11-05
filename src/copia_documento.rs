//! Prévia local de `ies documento salvar DOC` (#207;
//! contracts/cli/documento-salvar.md): o par `DOC.vN.html` (a foto que o
//! SEI serviu da versão N) e `DOC.v(N+1).html` (a edição local). A base é
//! a própria foto, conferida pelo SHA-256 que o índice local registrou ao
//! gravá-la; não há base nem manifesto ocultos. Nenhuma rede.
use crate::arquivo_copia::ler_regular;
use crate::arquivo_texto_documento::ErroArquivoTexto;
use crate::documento::NumeroDocumento;
use crate::documento_versoes::VersaoDocumento;
use crate::foto_documento::{nome, sha256};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroCopia {
    /// `--saida DIR` ausente, não diretório ou com ancestral simbólico.
    DiretorioDeSaida(ErroArquivoTexto),
    DestinoInseguro,
    LinkSimbolico,
    // Só construída na conferência de permissões Unix.
    #[cfg_attr(not(unix), allow(dead_code))]
    PermissoesInseguras,
    /// A foto local tem outro conteúdo que o servido (ou outro documento
    /// ou versão no próprio HTML).
    FotoDivergente,
    GravacaoFalhou,
    LeituraFalhou,
    NomeDaVersaoInvalido,
    VersaoDuplicada,
    BaseAusente,
    BaseNaoRegistrada,
    BaseDivergente,
    EdicaoAusente,
    EdicaoIgualABase,
}
impl ErroCopia {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::DiretorioDeSaida(erro) => erro.subcodigo(),
            Self::DestinoInseguro => "destino_copia_inseguro",
            Self::LinkSimbolico => "copia_local_link_simbolico",
            Self::PermissoesInseguras => "copia_local_permissoes_inseguras",
            Self::FotoDivergente => "foto_local_divergente",
            Self::GravacaoFalhou => "gravacao_foto_falhou",
            Self::LeituraFalhou => "leitura_copia_falhou",
            Self::NomeDaVersaoInvalido => "nome_versao_local_invalido",
            Self::VersaoDuplicada => "versao_local_repetida",
            Self::BaseAusente => "base_versao_ausente",
            Self::BaseNaoRegistrada => "base_versao_nao_registrada",
            Self::BaseDivergente => "base_versao_divergente",
            Self::EdicaoAusente => "versao_editada_ausente",
            Self::EdicaoIgualABase => "versao_editada_igual_a_base",
        }
    }
}

pub(crate) fn pasta_segura(pasta: &Path) -> Result<(), ErroCopia> {
    let dados = fs::symlink_metadata(pasta).map_err(|_| ErroCopia::DestinoInseguro)?;
    if dados.file_type().is_symlink() || !dados.is_dir() {
        return Err(ErroCopia::DestinoInseguro);
    }
    Ok(())
}

/// O SHA-256 que o índice local registrou para a foto `DOC.vN.html`.
pub(crate) type Registro<'a> = &'a dyn Fn(VersaoDocumento) -> Option<String>;

/// As versões `DOC.vN.html` presentes em `pasta`, em ordem numérica, sem
/// abrir os arquivos; duas grafias da mesma versão são ambíguas.
pub(crate) fn versoes_locais(
    pasta: &Path,
    documento: &NumeroDocumento,
) -> Result<BTreeSet<VersaoDocumento>, ErroCopia> {
    pasta_segura(pasta)?;
    let prefixo = format!("{}.v", documento.as_str());
    let mut versoes = BTreeSet::new();
    for item in fs::read_dir(pasta).map_err(|_| ErroCopia::LeituraFalhou)? {
        let item = item.map_err(|_| ErroCopia::LeituraFalhou)?;
        let nome_os = item.file_name();
        let Some(digitos) = nome_os
            .to_str()
            .and_then(|n| n.strip_prefix(&prefixo))
            .and_then(|n| n.strip_suffix(".html"))
        else {
            continue;
        };
        let versao =
            VersaoDocumento::interpretar(digitos).map_err(|_| ErroCopia::NomeDaVersaoInvalido)?;
        if !versoes.insert(versao) {
            return Err(ErroCopia::VersaoDuplicada);
        }
    }
    Ok(versoes)
}

/// Envelope contínuo mínimo de linhas potencialmente diferentes.
/// Índices zero-based e fim exclusivo; não é um diff por hunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TrechoDivergente {
    pub base_inicio: usize,
    pub base_fim_exclusivo: usize,
    pub trabalho_inicio: usize,
    pub trabalho_fim_exclusivo: usize,
}

/// Computa o menor intervalo CONTÍNUO que contém todas as diferenças
/// de linha. Não inventa diff fino nem armazena/expõe o texto.
fn trecho_divergente(base: &[u8], trabalho: &[u8]) -> Option<TrechoDivergente> {
    if base == trabalho {
        return None;
    }
    let total_base = base.split(|byte| *byte == b'\n').count();
    let total_trabalho = trabalho.split(|byte| *byte == b'\n').count();

    let mut prefixo = 0;
    let mut base_inicio = base.split(|byte| *byte == b'\n');
    let mut trabalho_inicio = trabalho.split(|byte| *byte == b'\n');
    loop {
        match (base_inicio.next(), trabalho_inicio.next()) {
            (Some(a), Some(b)) if a == b => prefixo += 1,
            _ => break,
        }
    }

    let mut sufixo = 0;
    let mut base_fim = base.rsplit(|byte| *byte == b'\n');
    let mut trabalho_fim = trabalho.rsplit(|byte| *byte == b'\n');
    while prefixo + sufixo < total_base && prefixo + sufixo < total_trabalho {
        match (base_fim.next(), trabalho_fim.next()) {
            (Some(a), Some(b)) if a == b => sufixo += 1,
            _ => break,
        }
    }
    Some(TrechoDivergente {
        base_inicio: prefixo,
        base_fim_exclusivo: total_base - sufixo,
        trabalho_inicio: prefixo,
        trabalho_fim_exclusivo: total_trabalho - sufixo,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreviaCopia {
    pub documento: String,
    /// N: a foto `DOC.vN.html`, conferida pelo SHA-256 registrado.
    pub versao_base: u32,
    /// N + 1: `DOC.v(N+1).html`, a edição local.
    pub versao_editada: u32,
    pub bytes_base: usize,
    pub bytes_editada: usize,
    pub escrita_remota: bool,
    pub trecho_divergente: Option<TrechoDivergente>,
}

/// O par a comparar: com `--versao N`, (N, N + 1); sem, a maior versão
/// local M é a edição (M - 1, M), salvo se M for uma foto registrada
/// com o mesmo SHA-256, quando a edição N + 1 ainda não existe.
fn par(
    pasta: &Path,
    documento: &NumeroDocumento,
    solicitada: Option<VersaoDocumento>,
    registro: Registro<'_>,
) -> Result<(VersaoDocumento, VersaoDocumento), ErroCopia> {
    let base = if let Some(n) = solicitada {
        n
    } else {
        let maior = *versoes_locais(pasta, documento)?
            .last()
            .ok_or(ErroCopia::BaseAusente)?;
        let bytes = ler_regular(
            &pasta.join(nome(documento.as_str(), maior)),
            ErroCopia::BaseAusente,
        )?;
        if registro(maior).is_some_and(|sha| sha == sha256(&bytes)) {
            maior
        } else {
            VersaoDocumento::nova(maior.numero() - 1).map_err(|_| ErroCopia::BaseAusente)?
        }
    };
    let editada = VersaoDocumento::nova(base.numero().saturating_add(1))
        .map_err(|_| ErroCopia::EdicaoAusente)?;
    Ok((base, editada))
}

/// Apenas prévia; nenhuma chamada Web, leitura do editor ou POST. Nunca
/// afirma a versão atual do SEI.
pub fn previa_local(
    pasta: &Path,
    documento: &NumeroDocumento,
    solicitada: Option<VersaoDocumento>,
    registro: Registro<'_>,
) -> Result<PreviaCopia, ErroCopia> {
    pasta_segura(pasta)?;
    let (n, seguinte) = par(pasta, documento, solicitada, registro)?;
    let numero = documento.as_str();
    let base = ler_regular(&pasta.join(nome(numero, n)), ErroCopia::BaseAusente)?;
    match registro(n) {
        None => return Err(ErroCopia::BaseNaoRegistrada),
        Some(sha) if sha != sha256(&base) => return Err(ErroCopia::BaseDivergente),
        Some(_) => {}
    }
    let editada = ler_regular(
        &pasta.join(nome(numero, seguinte)),
        ErroCopia::EdicaoAusente,
    )?;
    if editada == base {
        return Err(ErroCopia::EdicaoIgualABase);
    }
    Ok(PreviaCopia {
        documento: numero.to_owned(),
        versao_base: n.numero(),
        versao_editada: seguinte.numero(),
        bytes_base: base.len(),
        bytes_editada: editada.len(),
        escrita_remota: false,
        trecho_divergente: trecho_divergente(&base, &editada),
    })
}

#[cfg(test)]
#[path = "copia_documento/tests.rs"]
mod tests_par;

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn localiza_troca_de_linha_sem_divulgar_texto() {
        let antes = b"<p>cabecalho</p>\n<p>ANTIGO SIGILOSO</p>\n<p>fim</p>";
        let depois = b"<p>cabecalho</p>\n<p>NOVO SIGILOSO</p>\n<p>fim</p>";
        let trecho = trecho_divergente(antes, depois).unwrap();
        assert_eq!(
            trecho,
            TrechoDivergente {
                base_inicio: 1,
                base_fim_exclusivo: 2,
                trabalho_inicio: 1,
                trabalho_fim_exclusivo: 2,
            }
        );
        let json = serde_json::to_string(&trecho).unwrap();
        assert!(!json.contains("SIGILOSO"));
        assert!(!json.contains("<p>"));
    }

    #[test]
    fn insercao_no_meio_tem_intervalo_vazio_na_base() {
        assert_eq!(
            trecho_divergente(b"a\nc", b"a\nb\nc"),
            Some(TrechoDivergente {
                base_inicio: 1,
                base_fim_exclusivo: 1,
                trabalho_inicio: 1,
                trabalho_fim_exclusivo: 2,
            })
        );
    }

    #[test]
    fn edicoes_disjuntas_geram_envelope_conservador() {
        assert_eq!(
            trecho_divergente(b"a\nb\nc\nd\ne", b"a\nB\nc\nD\ne"),
            Some(TrechoDivergente {
                base_inicio: 1,
                base_fim_exclusivo: 4,
                trabalho_inicio: 1,
                trabalho_fim_exclusivo: 4,
            })
        );
    }

    #[test]
    fn arquivo_inalterado_e_linhas_nao_utf8_tem_comportamento_definido() {
        assert_eq!(trecho_divergente(b"a\nb", b"a\nb"), None);
        assert_eq!(
            trecho_divergente(&[255, 10, 65], &[255, 10, 66]),
            Some(TrechoDivergente {
                base_inicio: 1,
                base_fim_exclusivo: 2,
                trabalho_inicio: 1,
                trabalho_fim_exclusivo: 2,
            })
        );
    }
}
