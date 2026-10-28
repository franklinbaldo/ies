//! Contexto técnico de uma instalação SEI: endereço HTTPS e versão declarada.
//! Não é cadastro de órgãos/unidades e não armazena credenciais.
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersaoSei(Vec<u32>);

impl VersaoSei {
    pub fn interpretar(valor: &str) -> Option<Self> {
        if valor.len() > 32 {
            return None;
        }
        let partes: Vec<u32> = valor
            .split('.')
            .map(|p| {
                if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
                    None
                } else {
                    p.parse().ok()
                }
            })
            .collect::<Option<_>>()?;
        (2..=4).contains(&partes.len()).then_some(Self(partes))
    }
}

impl fmt::Display for VersaoSei {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, parte) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ".")?;
            }
            write!(f, "{parte}")?;
        }
        Ok(())
    }
}

/// Uma instalação pode atender múltiplos órgãos; a unidade ativa
/// pertence à sessão e pode mudar sem mudar a instalação.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instalacao {
    host: String,
    base: String,
    versao: Option<VersaoSei>,
}

pub(crate) fn host_valido(host: &str) -> bool {
    host.len() <= 253
        && !host.is_empty()
        && host.split('.').all(|p| {
            !p.is_empty()
                && p.len() <= 63
                && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && p.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
                && p.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
        })
}

impl Instalacao {
    /// Arquivo privado: base HTTPS na primeira linha e, opcionalmente,
    /// `versao=5.0.4` na segunda. Versão é declarada, não detectada.
    pub fn interpretar(config: &str) -> Option<Self> {
        let mut linhas = config.lines();
        let url = linhas.next()?.trim();
        if url.len() > 512
            || !url.bytes().all(|b| (33..=126).contains(&b))
            || url
                .chars()
                .any(|c| matches!(c, '?' | '#' | '@' | '\\' | '%'))
        {
            return None;
        }
        let restante = url.strip_prefix("https://")?;
        let (host, caminho) = restante.split_once('/').unwrap_or((restante, ""));
        if !host_valido(host) || caminho.ends_with("//") {
            return None;
        }
        let base = caminho.strip_suffix('/').unwrap_or(caminho);
        if !base.is_empty()
            && !base.split('/').all(|segmento| {
                !segmento.is_empty()
                    && segmento != "."
                    && segmento != ".."
                    && segmento
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
            })
        {
            return None;
        }
        let versao = match linhas.next() {
            Some(linha) => Some(VersaoSei::interpretar(
                linha.trim().strip_prefix("versao=")?,
            )?),
            None => None,
        };
        if linhas.next().is_some() {
            return None;
        }
        Some(Self {
            host: host.to_ascii_lowercase(),
            base: base.to_owned(),
            versao,
        })
    }

    /// Mesmo host não é suficiente: caminhos distintos podem ser
    /// instalações distintas. Nenhuma versão concede capacidades.
    pub fn autoriza_controlador(&self, host: &str, caminho: &str) -> bool {
        if !host.eq_ignore_ascii_case(&self.host) {
            return false;
        }
        let esperado = if self.base.is_empty() {
            "controlador.php".to_owned()
        } else {
            format!("{}/controlador.php", self.base)
        };
        caminho == esperado
    }
}

impl fmt::Display for Instalacao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "https://{}", self.host)?;
        if !self.base.is_empty() {
            write!(f, "/{}", self.base)?;
        }
        if let Some(versao) = &self.versao {
            write!(f, " (versão SEI declarada: {versao})")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpreta_versoes_reais_sem_supor_semver() {
        for texto in ["4.0", "4.0.9", "4.1.5", "5.0.3", "5.0.4", "4.0.4.6"] {
            assert_eq!(VersaoSei::interpretar(texto).unwrap().to_string(), texto);
        }
        for texto in ["", "5", "v5.0.4", "5.0.4-dev", "5..4", "4.1.2.3.4", "5.é.1"] {
            assert!(VersaoSei::interpretar(texto).is_none(), "{texto}");
        }
    }

    #[test]
    fn separa_instalacoes_no_mesmo_host() {
        let a = Instalacao::interpretar("https://sei.exemplo/um/sei\nversao=4.1.5").unwrap();
        let b = Instalacao::interpretar("https://sei.exemplo/dois/sei\nversao=5.0.4").unwrap();
        assert!(a.autoriza_controlador("sei.exemplo", "um/sei/controlador.php"));
        assert!(!a.autoriza_controlador("sei.exemplo", "dois/sei/controlador.php"));
        assert!(b.autoriza_controlador("SEI.EXEMPLO", "dois/sei/controlador.php"));
        assert!(!b.autoriza_controlador("outro.exemplo", "dois/sei/controlador.php"));
        assert!(a.to_string().contains("4.1.5"));
        assert!(b.to_string().contains("5.0.4"));
    }

    #[test]
    fn instala_na_raiz_nao_autoriza_outras_pastas() {
        let instalacao = Instalacao::interpretar("https://sei.exemplo/").unwrap();
        assert!(instalacao.autoriza_controlador("sei.exemplo", "controlador.php"));
        assert!(!instalacao.autoriza_controlador("sei.exemplo", "sei/controlador.php"));
        assert_eq!(instalacao.versao, None);
    }

    #[test]
    fn rejeita_configuracoes_ambiquas() {
        for texto in [
            "http://sei.exemplo/sei",
            "https://usuario@sei.exemplo/sei",
            "https://sei.exemplo/sei?token=segredo",
            "https://sei.exemplo/sei#fragmento",
            "https://sei.exemplo/../sei",
            "https://sei.exemplo/sei//outra",
            "https://sei.exemplo/sei//",
            "https://sei.exemplo/sei\nversao=5.0.4-dev",
            "https://sei.exemplo/sei\nversao=5.0.4\noutra=1",
        ] {
            assert!(Instalacao::interpretar(texto).is_none(), "{texto}");
        }
    }
}
