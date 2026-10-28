---
type: CommandContract
title: Listar processos da unidade
operation: unidade.listar_processos
command: ies unidade listar-processos
effect: read
implementation: stub
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Listar processos da unidade

`ies unidade listar-processos` lê as tabelas `Recebidos` e `Gerados`
na caixa da unidade ativa. A CLI de domínio ainda não tem
configuração canônica de conexão; retorna `nao_implementado`.

## Entradas

- `--saida ARQ`: destino de saída reservado para implementação futura.
- `--json` e `--unidade SIGLA`: opções do contrato público.

A reprodução das cassettes foi removida deste comando: consulte
[`ies replay CENARIO`](../../docs/replay.md). A leitura experimental de
sessão já autenticada está em `ies sessao ler-caixa` e exige origem
autorizada e arquivos privados; não se deve confundi-la com uma
conexão SEI configurada do usuário.

## Resultado de domínio e contrato JSON v1

O domínio tipado guarda apenas fatos: `Caixa` possui duas
`Tabela<Processo>`, cada qual com `processos` (id interno validado,
número e visualização) e `total_declarado: Option<usize>`.

A paginação verifica intervalos e identidades, sem armazenar
`completa`, contagens redundantes ou uma enumeração de completude
dentro do domínio.

O JSON v1 expõe `unidade`, `origem`, `recebidos` e `gerados`.
Cada tabela inclui apenas `processos`, `total_declarado` e
`completa: bool`, **calculado a partir dos intervalos verificados**.
`false` quer dizer cobertura não comprovada, não necessariamente
incompletude definitiva. `true` certifica somente cobertura dos
intervalos declarados, totais coerentes e IDs sem duplicação.
**Não demonstra snapshot transacional**, nem estabilidade dos
processos entre as requisições. Não existem no novo contrato os campos
antigos `completude`, `linhas_lidas`, `paginas_lidas`, `avisos`
e `fonte`. A saída humana calcula seus avisos separadamente.

O [golden JSON v1](../../tests/fixtures/caixa-v1.json)
protege o formato público. Não há campo `href` no `Processo`
para permitir serializar URLs assinadas inadvertidamente.

## Segurança e verificação

O HTTP permite somente GET HTTPS da ação `procedimento_controlar`
publicada com `infra_hash` e host idêntico à origem autorizada;
não segue redirects nem faz retries. O corpo HTTP é limitado a
8 MB **durante o recebimento**, incluindo respostas sem
`Content-Length`. Redirecionamento, falta de autorização e
origem incompatível produzem subcódigos seguros.
O modo offline confere os hashes do ZIP e do JSON, schema e origem.
O parser recusa tabela ausente, ID repetido e número ambíguo.

Esta fase **consolida páginas offline**, exigindo faixas contíguas,
totais coerentes e IDs distintos por categoria. Para juntar páginas,
exige unidade exibida, modo de visualização e hashes dos critérios
iguais nos HTMLs. Divergência recusa a junção; ausência de marcador
impede confirmar a mescla. Os campos ocultos não são publicados
no JSON e não são prova de identidade de sessão. Para juntar páginas,
exige unidade exibida, modo de visualização e hashes dos critérios
iguais nos HTMLs. Divergência recusa a junção; ausência de marcador
impede confirmar a mescla. Os campos ocultos não são publicados
no JSON e não são prova de identidade de sessão. A paginação HTTP
continua bloqueada: o corpus histórico não preserva os corpos de POST
necessários para reproduzir esse gesto com segurança. Se só a primeira
página foi lida, a saída continua explicitamente parcial.

[Evidências](../evidencias/unidade-listar_processos.md),
[replay](../../docs/cassettes.md) e
[como o SEI funciona](../../docs/como-o-sei-funciona/tabelas-e-paginacao.md)
sustentam este recorte. Ainda não há equivalência funcional plena
com a CLI do Todos.
