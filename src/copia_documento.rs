//! Cópias locais imutáveis por geração; não é transporte de escrita no SEI.
use crate::arquivo_copia::ler_regular;
use crate::documento::NumeroDocumento;
use crate::documento_versoes::VersaoDocumento;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroCopia {
    FonteNaoEditavel,
    DocumentoAssinado,
    IdentidadeInvalida,
    DestinoInseguro,
    LinkSimbolico,
    PermissoesInseguras,
    ArquivoExistente,
    ArquivoAusente,
    BaseAusente,
    BaseDivergente,
    IdentidadeDivergente,
    GravacaoFalhou,
    LeituraFalhou,
}
impl ErroCopia {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::FonteNaoEditavel => "fonte_html_nao_editavel",
            Self::DocumentoAssinado => "documento_assinado_editor_proibido",
            Self::IdentidadeInvalida => "identidade_copia_invalida",
            Self::DestinoInseguro => "destino_copia_inseguro",
            Self::LinkSimbolico => "copia_local_link_simbolico",
            Self::PermissoesInseguras => "copia_local_permissoes_inseguras",
            Self::ArquivoExistente => "copia_versao_ja_existe",
            Self::ArquivoAusente => "copia_versao_ausente",
            Self::BaseAusente => "base_versao_ausente",
            Self::BaseDivergente => "base_versao_divergente",
            Self::IdentidadeDivergente => "identidade_copia_divergente",
            Self::GravacaoFalhou => "gravacao_copia_falhou",
            Self::LeituraFalhou => "leitura_copia_falhou",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ManifestoCopia {
    documento: String,
    id_interno: String,
    instalacao: String,
    versao: u32,
    hash_base: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentidadeCopia {
    pub documento: NumeroDocumento,
    pub id_interno: String,
    pub instalacao: String,
    pub versao: VersaoDocumento,
}
impl IdentidadeCopia {
    fn conferir(&self) -> Result<(), ErroCopia> {
        if self.id_interno.is_empty()
            || !self.id_interno.bytes().all(|b| b.is_ascii_digit())
            || !self.instalacao.starts_with("https://")
            || self.instalacao.len() > 512
            || self
                .instalacao
                .chars()
                .any(|c| matches!(c, '?' | '#' | '@' | '\\'))
        {
            return Err(ErroCopia::IdentidadeInvalida);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FonteHtml<'a> {
    /// Só deve ser produzida por adaptador após observar documento não assinado,
    /// com editor que comprove as seções realmente editáveis.
    EditorVerificado {
        html: &'a str,
        assinado: bool,
    },
    VisualizacaoHistorica,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaminhosCopia {
    pub trabalho: PathBuf,
    pub base: PathBuf,
    pub manifesto: PathBuf,
}
pub fn caminhos(pasta: &Path, id: &IdentidadeCopia) -> CaminhosCopia {
    let nome = format!("{}.v{}", id.documento.as_str(), id.versao.numero());
    CaminhosCopia {
        trabalho: pasta.join(format!("{nome}.html")),
        base: pasta.join(format!(".{nome}.base.html")),
        manifesto: pasta.join(format!(".{nome}.meta.json")),
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn pasta_segura(pasta: &Path) -> Result<(), ErroCopia> {
    let dados = fs::symlink_metadata(pasta).map_err(|_| ErroCopia::DestinoInseguro)?;
    if dados.file_type().is_symlink() || !dados.is_dir() {
        return Err(ErroCopia::DestinoInseguro);
    }
    Ok(())
}
fn conferir_ausencia(caminhos: &CaminhosCopia) -> Result<(), ErroCopia> {
    for alvo in [&caminhos.trabalho, &caminhos.base, &caminhos.manifesto] {
        if fs::symlink_metadata(alvo).is_ok() {
            return Err(ErroCopia::ArquivoExistente);
        }
    }
    Ok(())
}
fn criar_novo(alvo: &Path, bytes: &[u8]) -> Result<(), ErroCopia> {
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts.open(alvo).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            ErroCopia::ArquivoExistente
        } else {
            ErroCopia::GravacaoFalhou
        }
    })?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| ErroCopia::GravacaoFalhou)
}
fn manifesto(id: &IdentidadeCopia, base: &[u8]) -> ManifestoCopia {
    ManifestoCopia {
        documento: id.documento.as_str().to_owned(),
        id_interno: id.id_interno.clone(),
        instalacao: id.instalacao.clone(),
        versao: id.versao.numero(),
        hash_base: digest(base),
    }
}
/// Publicação sem sobrescrita: em falha remove apenas os arquivos criados por
/// esta chamada; um processo interrompido pode deixar par órfão, detectado.
pub fn preparar(
    pasta: &Path,
    id: &IdentidadeCopia,
    fonte: FonteHtml<'_>,
) -> Result<CaminhosCopia, ErroCopia> {
    id.conferir()?;
    pasta_segura(pasta)?;
    let html = match fonte {
        FonteHtml::EditorVerificado {
            html,
            assinado: false,
        } => html.as_bytes(),
        FonteHtml::EditorVerificado { assinado: true, .. } => {
            return Err(ErroCopia::DocumentoAssinado);
        }
        FonteHtml::VisualizacaoHistorica => return Err(ErroCopia::FonteNaoEditavel),
    };
    if html.is_empty() || html.len() > 8_000_000 {
        return Err(ErroCopia::FonteNaoEditavel);
    }
    let paths = caminhos(pasta, id);
    conferir_ausencia(&paths)?;
    let meta = serde_json::to_vec(&manifesto(id, html)).map_err(|_| ErroCopia::GravacaoFalhou)?;
    let mut criados = Vec::new();
    for (alvo, bytes) in [
        (&paths.base, html),
        (&paths.manifesto, meta.as_slice()),
        (&paths.trabalho, html),
    ] {
        match criar_novo(alvo, bytes) {
            Ok(()) => criados.push(alvo),
            Err(erro) => {
                for criado in criados {
                    let _ = fs::remove_file(criado);
                }
                return Err(erro);
            }
        }
    }
    Ok(paths)
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
    pub versao_base: u32,
    pub alterado: bool,
    pub bytes_base: usize,
    pub bytes_trabalho: usize,
    pub escrita_remota: bool,
    pub trecho_divergente: Option<TrechoDivergente>,
}
/// Apenas prévia; nenhuma chamada Web, leitura do editor ou POST.
pub fn previa(pasta: &Path, id: &IdentidadeCopia) -> Result<PreviaCopia, ErroCopia> {
    id.conferir()?;
    pasta_segura(pasta)?;
    let paths = caminhos(pasta, id);
    let trabalho = ler_regular(&paths.trabalho, ErroCopia::ArquivoAusente)?;
    let base = ler_regular(&paths.base, ErroCopia::BaseAusente)?;
    let metadata = ler_regular(&paths.manifesto, ErroCopia::BaseAusente)?;
    let observado: ManifestoCopia =
        serde_json::from_slice(&metadata).map_err(|_| ErroCopia::BaseDivergente)?;
    if observado.documento != id.documento.as_str()
        || observado.id_interno != id.id_interno
        || observado.instalacao != id.instalacao
        || observado.versao != id.versao.numero()
    {
        return Err(ErroCopia::IdentidadeDivergente);
    }
    if observado.hash_base != digest(&base) {
        return Err(ErroCopia::BaseDivergente);
    }
    Ok(PreviaCopia {
        documento: id.documento.as_str().to_owned(),
        versao_base: id.versao.numero(),
        alterado: trabalho != base,
        bytes_base: base.len(),
        bytes_trabalho: trabalho.len(),
        escrita_remota: false,
        trecho_divergente: trecho_divergente(&base, &trabalho),
    })
}

/// Seleciona cópia local sem consultar o histórico remoto. Não afirma versão
/// atual do SEI e nunca tenta publicar nem repetir uma escrita.
pub fn previa_local(
    pasta: &Path,
    documento: &NumeroDocumento,
    solicitada: Option<VersaoDocumento>,
) -> Result<PreviaCopia, ErroCopia> {
    pasta_segura(pasta)?;
    let versao = if let Some(n) = solicitada {
        n
    } else {
        let mut maior = None;
        let prefixo = format!("{}.v", documento.as_str());
        for item in fs::read_dir(pasta).map_err(|_| ErroCopia::LeituraFalhou)? {
            let item = item.map_err(|_| ErroCopia::LeituraFalhou)?;
            let nome = item.file_name();
            let nome = nome.to_string_lossy();
            if let Some(n) = nome
                .strip_prefix(&prefixo)
                .and_then(|n| n.strip_suffix(".html"))
                && let Ok(n) = VersaoDocumento::interpretar(n)
            {
                maior = Some(maior.map_or(n, |atual: VersaoDocumento| atual.max(n)));
            }
        }
        maior.ok_or(ErroCopia::ArquivoAusente)?
    };
    let prefixo = format!(".{}.v{}.meta.json", documento.as_str(), versao.numero());
    let metadata = ler_regular(&pasta.join(prefixo), ErroCopia::BaseAusente)?;
    let observado: ManifestoCopia =
        serde_json::from_slice(&metadata).map_err(|_| ErroCopia::BaseDivergente)?;
    let identidade = IdentidadeCopia {
        documento: documento.clone(),
        id_interno: observado.id_interno,
        instalacao: observado.instalacao,
        versao,
    };
    previa(pasta, &identidade)
}

#[cfg(test)]
mod testes {
    use super::*;
    fn contexto() -> IdentidadeCopia {
        IdentidadeCopia {
            documento: NumeroDocumento::novo("0011519").unwrap(),
            id_interno: "42".into(),
            instalacao: "https://sei.exemplo/sei".into(),
            versao: VersaoDocumento::nova(4).unwrap(),
        }
    }
    fn diretorio() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ies-versoes-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn nao_sobrescreve_versoes_nem_edicoes_locais() {
        let pasta = diretorio();
        let id = contexto();
        let fonte = FonteHtml::EditorVerificado {
            html: "<p>original</p>",
            assinado: false,
        };
        let paths = preparar(&pasta, &id, fonte).unwrap();
        assert!(!previa(&pasta, &id).unwrap().alterado);
        fs::write(&paths.trabalho, "<p>editado</p>").unwrap();
        assert!(previa(&pasta, &id).unwrap().alterado);
        assert_eq!(
            preparar(&pasta, &id, fonte),
            Err(ErroCopia::ArquivoExistente)
        );
        assert_eq!(fs::read(&paths.base).unwrap(), b"<p>original</p>");
        fs::remove_dir_all(pasta).unwrap();
    }
    #[test]
    fn recusa_assinado_historico_base_ausente_e_identidade_trocada() {
        let pasta = diretorio();
        let mut id = contexto();
        assert_eq!(
            preparar(&pasta, &id, FonteHtml::VisualizacaoHistorica),
            Err(ErroCopia::FonteNaoEditavel)
        );
        assert_eq!(
            preparar(
                &pasta,
                &id,
                FonteHtml::EditorVerificado {
                    html: "x",
                    assinado: true
                }
            ),
            Err(ErroCopia::DocumentoAssinado)
        );
        let paths = preparar(
            &pasta,
            &id,
            FonteHtml::EditorVerificado {
                html: "x",
                assinado: false,
            },
        )
        .unwrap();
        id.id_interno = "43".into();
        assert_eq!(previa(&pasta, &id), Err(ErroCopia::IdentidadeDivergente));
        fs::remove_file(&paths.base).unwrap();
        assert_eq!(previa(&pasta, &id), Err(ErroCopia::BaseAusente));
        fs::remove_dir_all(pasta).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn nao_segue_link_simbolico() {
        use std::os::unix::fs::symlink;
        let pasta = diretorio();
        let id = contexto();
        let paths = caminhos(&pasta, &id);
        symlink("/etc/passwd", &paths.trabalho).unwrap();
        assert_eq!(
            preparar(
                &pasta,
                &id,
                FonteHtml::EditorVerificado {
                    html: "x",
                    assinado: false
                }
            ),
            Err(ErroCopia::ArquivoExistente)
        );
        assert_eq!(previa(&pasta, &id), Err(ErroCopia::LinkSimbolico));
        fs::remove_dir_all(pasta).unwrap();
    }

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

    #[test]
    fn previa_publica_trecho_somente_depois_de_conferir_a_base() {
        let pasta = diretorio();
        let id = contexto();
        let paths = preparar(
            &pasta,
            &id,
            FonteHtml::EditorVerificado {
                html: "cabecalho\nantigo\nfim",
                assinado: false,
            },
        )
        .unwrap();
        assert_eq!(previa(&pasta, &id).unwrap().trecho_divergente, None);
        fs::write(&paths.trabalho, "cabecalho\nnovo\nfim").unwrap();
        let resultado = previa(&pasta, &id).unwrap();
        assert!(resultado.alterado);
        assert_eq!(resultado.trecho_divergente.unwrap().base_inicio, 1);
        fs::remove_dir_all(pasta).unwrap();
    }
}
