//! Normalização de um fragmento de corpo com o parser e serializador HTML5.
use super::{Conteudo, Documento, fragmento};
use crate::tela_html::HtmlServido;
use html5ever::QualName;
use html5ever::serialize::{Serialize, SerializeOpts, Serializer, TraversalScope, serialize};
use std::io;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroNormalizacao {
    EntradaAcimaDoLimite(#[from] crate::tela_html::HtmlAcimaDoLimite),
    Serializacao(#[from] io::Error),
    Utf8(#[source] std::str::Utf8Error),
    FragmentoPerdidoNaNormalizacao,
    SaidaAcimaDoLimite,
}

pub(crate) fn normalizar_fragmento(html: &str) -> Result<String, ErroNormalizacao> {
    let html = HtmlServido::novo(html.to_owned())?;
    let mut bytes = Vec::new();
    serialize(&mut bytes, &fragmento(&html), SerializeOpts::default())?;
    if bytes.len() > crate::MAX_HTML_BYTES {
        return Err(ErroNormalizacao::SaidaAcimaDoLimite);
    }
    let normalizado =
        String::from_utf8(bytes).map_err(|erro| ErroNormalizacao::Utf8(erro.utf8_error()))?;
    if !html.trim().is_empty() && normalizado.trim().is_empty() {
        return Err(ErroNormalizacao::FragmentoPerdidoNaNormalizacao);
    }
    Ok(normalizado)
}

enum Evento<'a> {
    Abrir(usize),
    Fechar(&'a QualName),
}

impl Serialize for Documento {
    fn serialize<S: Serializer>(
        &self,
        serializador: &mut S,
        _escopo: TraversalScope,
    ) -> io::Result<()> {
        let mut eventos = self
            .filhos_invertidos(self.raiz)
            .into_iter()
            .map(Evento::Abrir)
            .collect::<Vec<_>>();
        while let Some(evento) = eventos.pop() {
            let indice = match evento {
                Evento::Abrir(indice) => indice,
                Evento::Fechar(nome) => {
                    serializador.end_elem(nome.clone())?;
                    continue;
                }
            };
            let Some(no) = self.nos.get(indice) else {
                continue;
            };
            match &no.conteudo {
                Conteudo::Raiz => {
                    eventos.extend(no.filhos.iter().rev().copied().map(Evento::Abrir))
                }
                Conteudo::Texto(texto) => serializador.write_text(texto)?,
                Conteudo::Comentario(texto) => serializador.write_comment(texto)?,
                Conteudo::Elemento(dados) => {
                    serializador.start_elem(
                        dados.nome.clone(),
                        dados.atributos.iter().map(|a| (&a.name, a.value.as_ref())),
                    )?;
                    eventos.push(Evento::Fechar(&dados.nome));
                    let filhos = dados
                        .conteudo_do_modelo
                        .and_then(|raiz| self.nos.get(raiz))
                        .map_or(&no.filhos, |modelo| &modelo.filhos);
                    eventos.extend(filhos.iter().rev().copied().map(Evento::Abrir));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn fecha_tags_sem_remover_estilos_ou_comentarios() {
        let original = "<!--nota--><p style='color:red'>A\r\n<strong>B";
        assert_eq!(
            normalizar_fragmento(original).unwrap(),
            "<!--nota--><p style=\"color:red\">A\n<strong>B</strong></p>"
        );
    }

    #[test]
    fn preserva_o_conteudo_de_elementos_template() {
        assert_eq!(
            normalizar_fragmento("<template><b>Modelo</b></template>").unwrap(),
            "<template><b>Modelo</b></template>"
        );
    }

    #[test]
    fn nao_converte_marcacao_incompleta_em_edicao_vazia() {
        assert!(matches!(
            normalizar_fragmento("<p title=\"aberto"),
            Err(ErroNormalizacao::FragmentoPerdidoNaNormalizacao)
        ));
    }
}
