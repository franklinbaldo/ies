//! `#[derive(Causa)]`: a identidade pública de um erro sai da própria
//! variante, declarada uma vez, ao lado de onde o erro é levantado.
//!
//! - `#[causa(interna)]`, numa variante de tupla com um campo, compõe: o
//!   caminho continua no erro interno e a ação e o status vêm dele.
//! - `#[causa(fonte)]`, numa variante de tupla com um campo, guarda a
//!   causa que a levantou: o caminho continua nela, a ação e o status vêm
//!   dela, mas a variante segue sendo o fato (a `folha`). Várias variantes
//!   podem ter a mesma fonte; não há `From`.
//! - `#[causa(transparente)]`: como `interna`, mas a variante não entra no
//!   caminho; o tipo interno já se nomeia (`Html(ErroHtml)` → `html/...`).
//! - `#[causa(http)]`, numa variante de tupla com um `StatusHttpRecusado`,
//!   publica status e mensagem do SEI.
//! - `#[causa(sessao_encerrada)]` marca o fato "a sessão do SEI acabou":
//!   `sessao_encerrada()` decide o relogin pelo tipo, sem comparar texto;
//!   as composições perguntam ao erro interno.
//!
//! Sem `syn`/`quote`: lê só os tokens do `enum`, e o código gerado é texto.
use proc_macro::{Delimiter, TokenStream, TokenTree};
use std::fmt::Write as _;

#[derive(Default)]
struct Atributos {
    sessao_encerrada: bool,
    ligacoes: Vec<Ligacao>,
}

/// Como a variante se liga ao campo: no máximo uma por variante.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ligacao {
    Interna,
    Transparente,
    Fonte,
    Http,
}

impl Atributos {
    fn tem(&self, ligacao: Ligacao) -> bool {
        self.ligacoes.contains(&ligacao)
    }
    /// `transparente` compõe como `interna`.
    fn interna(&self) -> bool {
        self.tem(Ligacao::Interna) || self.tem(Ligacao::Transparente)
    }
}

enum Forma {
    Unidade,
    Tupla(usize),
    Registro,
}

struct Variante {
    ident: String,
    forma: Forma,
    atributos: Atributos,
    /// O tipo do campo, numa tupla de um campo.
    campo: Option<String>,
}

/// Gera `impl crate::erro_publico::Causa`.
///
/// # Panics
///
/// Só se o próprio gerador produzir código que não tokeniza (defeito do
/// derive, não do `enum`).
#[proc_macro_derive(Causa, attributes(causa))]
pub fn derive_causa(entrada: TokenStream) -> TokenStream {
    match gerar(entrada) {
        Ok(codigo) => codigo.parse().expect("código gerado válido"),
        Err(erro) => format!("compile_error!({erro:?});").parse().expect("erro"),
    }
}

fn gerar(entrada: TokenStream) -> Result<String, String> {
    let mut tokens = entrada.into_iter().peekable();
    let mut nome = None;
    while let Some(token) = tokens.next() {
        if let TokenTree::Ident(ident) = &token
            && ident.to_string() == "enum"
        {
            match tokens.next() {
                Some(TokenTree::Ident(n)) => nome = Some(n.to_string()),
                _ => return Err("Causa: nome do enum ausente".into()),
            }
            break;
        }
    }
    let nome = nome.ok_or("Causa: só para enum")?;
    let corpo = match tokens.next() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g.stream(),
        _ => {
            return Err(format!(
                "Causa: {nome} com genéricos ou sem corpo não é suportado"
            ));
        }
    };
    let variantes = ler_variantes(corpo)?;
    if variantes.is_empty() {
        return Err(format!("Causa: {nome} sem variantes"));
    }
    Ok(escrever(&nome, &variantes))
}

fn ler_variantes(corpo: TokenStream) -> Result<Vec<Variante>, String> {
    let mut variantes = Vec::new();
    let mut atributos = Atributos::default();
    let mut tokens = corpo.into_iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                if let Some(TokenTree::Group(g)) = tokens.next() {
                    ler_atributo(g.stream(), &mut atributos)?;
                }
            }
            TokenTree::Ident(ident) => {
                let mut campo = None;
                let forma = match tokens.peek() {
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                        let campos = contar_campos(g.stream());
                        if campos == 1 {
                            campo = Some(g.stream().to_string());
                        }
                        tokens.next();
                        Forma::Tupla(campos)
                    }
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                        tokens.next();
                        Forma::Registro
                    }
                    _ => Forma::Unidade,
                };
                // Pula discriminante e a vírgula.
                for resto in tokens.by_ref() {
                    if matches!(&resto, TokenTree::Punct(p) if p.as_char() == ',') {
                        break;
                    }
                }
                let variante = Variante {
                    ident: ident.to_string(),
                    forma,
                    atributos: std::mem::take(&mut atributos),
                    campo,
                };
                validar(&variante)?;
                variantes.push(variante);
            }
            _ => {}
        }
    }
    Ok(variantes)
}

fn contar_campos(campos: TokenStream) -> usize {
    let mut profundidade = 0usize;
    let mut virgulas = 0;
    let mut algum = false;
    for token in campos {
        algum = true;
        if let TokenTree::Punct(p) = &token {
            match p.as_char() {
                '<' => profundidade += 1,
                '>' => profundidade = profundidade.saturating_sub(1),
                ',' if profundidade == 0 => virgulas += 1,
                _ => {}
            }
        }
    }
    usize::from(algum) + virgulas
}

fn ler_atributo(atributo: TokenStream, destino: &mut Atributos) -> Result<(), String> {
    let mut tokens = atributo.into_iter();
    let Some(TokenTree::Ident(nome)) = tokens.next() else {
        return Ok(());
    };
    if nome.to_string() != "causa" {
        return Ok(());
    }
    let Some(TokenTree::Group(argumentos)) = tokens.next() else {
        return Err("Causa: use #[causa(...)]".into());
    };
    for token in argumentos.stream() {
        let TokenTree::Ident(chave) = token else {
            continue;
        };
        match chave.to_string().as_str() {
            "interna" => destino.ligacoes.push(Ligacao::Interna),
            "transparente" => destino.ligacoes.push(Ligacao::Transparente),
            "fonte" => destino.ligacoes.push(Ligacao::Fonte),
            "http" => destino.ligacoes.push(Ligacao::Http),
            "sessao_encerrada" => destino.sessao_encerrada = true,
            outra => return Err(format!("Causa: argumento desconhecido `{outra}`")),
        }
    }
    Ok(())
}

fn validar(v: &Variante) -> Result<(), String> {
    let um_campo = matches!(v.forma, Forma::Tupla(1));
    let ligados = &v.atributos.ligacoes;
    if !ligados.is_empty() && !um_campo {
        return Err(format!(
            "Causa: {} com interna/transparente/fonte/http precisa ser tupla de um campo",
            v.ident
        ));
    }
    if ligados.len() > 1 {
        return Err(format!(
            "Causa: {} aceita só um de interna/transparente/fonte/http",
            v.ident
        ));
    }
    Ok(())
}

fn padrao(v: &Variante, ligar: bool) -> String {
    match v.forma {
        Forma::Unidade => format!("Self::{}", v.ident),
        Forma::Tupla(1) if ligar => format!("Self::{}(interna)", v.ident),
        Forma::Tupla(_) => format!("Self::{}(..)", v.ident),
        Forma::Registro => format!("Self::{} {{ .. }}", v.ident),
    }
}

fn escrever(nome: &str, variantes: &[Variante]) -> String {
    const T: &str = "crate::erro_publico";
    let mut http = String::new();
    let mut encerrada = String::new();
    for v in variantes {
        let compoe = v.atributos.interna() || v.atributos.tem(Ligacao::Fonte);
        let ligado = compoe || v.atributos.tem(Ligacao::Http);
        let p = padrao(v, ligado);
        if v.atributos.sessao_encerrada {
            let _ = writeln!(encerrada, "{} => true,", padrao(v, false));
        } else if compoe {
            let _ = writeln!(encerrada, "{p} => {T}::Causa::sessao_encerrada(interna),");
        }
        if v.atributos.tem(Ligacao::Http) {
            let _ = writeln!(http, "{p} => Some(interna),");
        } else if compoe {
            let _ = writeln!(http, "{p} => {T}::Causa::status_http(interna),");
        }
    }
    // `?` compõe: um `From` por tipo interno que aparece numa só variante.
    // Tipo repetido em duas variantes indica duas etapas achatadas num
    // nível: separe-as em tipos próprios em vez de escolher uma.
    let mut conversoes = String::new();
    for v in variantes.iter().filter(|v| v.atributos.interna()) {
        let Some(tipo) = &v.campo else { continue };
        let repetido = variantes
            .iter()
            .filter(|o| o.atributos.interna() && o.campo.as_ref() == Some(tipo))
            .count()
            > 1;
        if !repetido {
            let _ = writeln!(
                conversoes,
                "#[automatically_derived]\nimpl From<{tipo}> for {nome} {{ fn from(interna: {tipo}) -> Self {{ Self::{}(interna) }} }}",
                v.ident
            );
        }
    }
    format!(
        "{conversoes}
#[automatically_derived]
impl {T}::Causa for {nome} {{
    #[allow(unreachable_patterns, unused_variables)]
    fn status_http(&self) -> Option<&crate::transporte::StatusHttpRecusado> {{
        match self {{ {http} _ => None }}
    }}
    #[allow(unreachable_patterns, unused_variables)]
    fn sessao_encerrada(&self) -> bool {{
        match self {{ {encerrada} _ => false }}
    }}
}}"
    )
}
