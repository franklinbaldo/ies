//! `#[derive(Causa)]`: a identidade pública de um erro sai da própria
//! variante, declarada uma vez, ao lado de onde o erro é levantado.
//!
//! - O nome é o `snake_case` da variante (`LinkAusente` → `link_ausente`).
//! - `#[causa(nome = "...")]` registra uma exceção histórica.
//! - `#[causa(interna)]`, numa variante de tupla com um campo, compõe: o
//!   caminho continua no erro interno e a ação e o status vêm dele.
//! - `#[causa(acao = "relogar")]` fixa a ação de recuperação.
//! - `#[causa(http)]`, numa variante de tupla com um `StatusHttpRecusado`,
//!   publica status e mensagem do SEI.
//!
//! Sem `syn`/`quote`: lê só os tokens do `enum`, e o código gerado é texto.
use proc_macro::{Delimiter, TokenStream, TokenTree};
use std::fmt::Write as _;

#[derive(Default)]
struct Atributos {
    nome: Option<String>,
    acao: Option<String>,
    interna: bool,
    http: bool,
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
    let mut args = argumentos.stream().into_iter().peekable();
    while let Some(token) = args.next() {
        let TokenTree::Ident(chave) = token else {
            continue;
        };
        match chave.to_string().as_str() {
            "interna" => destino.interna = true,
            "http" => destino.http = true,
            chave @ ("nome" | "acao") => {
                let igual = args.next();
                let valor = args.next();
                let (Some(TokenTree::Punct(p)), Some(TokenTree::Literal(literal))) = (igual, valor)
                else {
                    return Err(format!("Causa: use {chave} = \"...\""));
                };
                if p.as_char() != '=' {
                    return Err(format!("Causa: use {chave} = \"...\""));
                }
                let texto = literal.to_string();
                let texto = texto
                    .strip_prefix('"')
                    .and_then(|t| t.strip_suffix('"'))
                    .ok_or_else(|| format!("Causa: {chave} precisa de string"))?
                    .to_owned();
                if chave == "nome" {
                    destino.nome = Some(texto);
                } else {
                    destino.acao = Some(texto);
                }
            }
            outra => return Err(format!("Causa: argumento desconhecido `{outra}`")),
        }
    }
    Ok(())
}

fn validar(v: &Variante) -> Result<(), String> {
    let um_campo = matches!(v.forma, Forma::Tupla(1));
    if (v.atributos.interna || v.atributos.http) && !um_campo {
        return Err(format!(
            "Causa: {} com interna/http precisa ser tupla de um campo",
            v.ident
        ));
    }
    if v.atributos.interna && v.atributos.http {
        return Err(format!("Causa: {} não pode ser interna e http", v.ident));
    }
    if let Some(nome) = &v.atributos.nome
        && (nome.is_empty()
            || !nome
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'))
    {
        return Err(format!("Causa: nome inválido em {}", v.ident));
    }
    Ok(())
}

/// `StatusHttpDiferenteDe200` → `status_http_diferente_de_200`;
/// `DuranteALeitura` → `durante_a_leitura`.
fn snake(ident: &str) -> String {
    let c: Vec<char> = ident.chars().collect();
    let mut saida = String::new();
    for (i, &atual) in c.iter().enumerate() {
        if i > 0 {
            let antes = c[i - 1];
            let depois = c.get(i + 1).copied();
            let maiuscula = atual.is_ascii_uppercase()
                && (!antes.is_ascii_uppercase() || depois.is_some_and(|d| d.is_ascii_lowercase()));
            let numero = atual.is_ascii_digit() && !antes.is_ascii_digit();
            if maiuscula || numero {
                saida.push('_');
            }
        }
        saida.push(atual.to_ascii_lowercase());
    }
    saida
}

/// `corrigir_argumentos` → `CorrigirArgumentos`.
fn camel(texto: &str) -> String {
    texto
        .split('_')
        .map(|parte| {
            let mut c = parte.chars();
            c.next()
                .map(|p| p.to_ascii_uppercase().to_string() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
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
    let mut caminho = String::new();
    let mut acao = String::new();
    let mut http = String::new();
    for v in variantes {
        let rotulo = v.atributos.nome.clone().unwrap_or_else(|| snake(&v.ident));
        let ligado = v.atributos.interna || v.atributos.http;
        let p = padrao(v, ligado);
        if v.atributos.interna {
            let _ = writeln!(
                caminho,
                "{p} => {{ c.push({rotulo:?}); {T}::Causa::caminho(interna, c) }}"
            );
        } else {
            let _ = writeln!(caminho, "{p} => c.push({rotulo:?}),");
        }
        let fixa = v
            .atributos
            .acao
            .as_ref()
            .map(|a| format!("{T}::Acao::{}", camel(a)));
        let a = match (fixa, v.atributos.interna) {
            (Some(fixa), _) => fixa,
            (None, true) => format!("{T}::Causa::acao(interna)"),
            (None, false) => format!("{T}::Acao::Reportar"),
        };
        let _ = writeln!(acao, "{} => {a},", padrao(v, v.atributos.interna));
        if v.atributos.http {
            let _ = writeln!(http, "{p} => Some(interna),");
        } else if v.atributos.interna {
            let _ = writeln!(http, "{p} => {T}::Causa::status_http(interna),");
        }
    }
    // `?` compõe: um `From` por tipo interno que aparece numa só variante.
    // Tipo repetido em duas variantes indica duas etapas achatadas num
    // nível: separe-as em tipos próprios em vez de escolher uma.
    let mut conversoes = String::new();
    for v in variantes.iter().filter(|v| v.atributos.interna) {
        let Some(tipo) = &v.campo else { continue };
        let repetido = variantes
            .iter()
            .filter(|o| o.atributos.interna && o.campo.as_ref() == Some(tipo))
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
    fn caminho(&self, c: &mut Vec<&'static str>) {{
        match self {{ {caminho} }}
    }}
    #[allow(unused_variables)]
    fn acao(&self) -> {T}::Acao {{
        match self {{ {acao} }}
    }}
    #[allow(unreachable_patterns, unused_variables)]
    fn status_http(&self) -> Option<&crate::transporte::StatusHttpRecusado> {{
        match self {{ {http} _ => None }}
    }}
}}"
    )
}

#[cfg(test)]
mod tests {
    use super::{camel, snake};

    #[test]
    fn snake_separa_maiusculas_e_numeros() {
        assert_eq!(snake("LinkAusente"), "link_ausente");
        assert_eq!(
            snake("StatusHttpDiferenteDe200"),
            "status_http_diferente_de_200"
        );
        assert_eq!(snake("Rede"), "rede");
        assert_eq!(
            snake("SessaoTrocadaDuranteALeitura"),
            "sessao_trocada_durante_a_leitura"
        );
        assert_eq!(snake("ParaOLogin"), "para_o_login");
    }

    #[test]
    fn camel_monta_a_variante_da_acao() {
        assert_eq!(camel("corrigir_argumentos"), "CorrigirArgumentos");
        assert_eq!(camel("relogar"), "Relogar");
    }
}
