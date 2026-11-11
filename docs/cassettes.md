---
type: Guide
title: Catálogo das cassettes públicas
description: Localizar, conferir e reproduzir gravações de treinamento do SEI
---

# Cassettes do SEI

O catálogo `cassettes/` é **um único bundle OKF**, com um arquivo
Markdown para cada revisão distinta de cassette. O Git guarda apenas
os metadados verificáveis: todo ZIP e toda cassette JSON original
continuam no [Internet Archive](https://archive.org/details/todos-sei-cassettes-2026).

O inventário atual contém **29 ZIPs (7 históricos e 22 lotes)**, com
502 capturas e **495 revisões distintas**; sete cópias idênticas
foram consolidadas. Instalações: UnB, UFRJ, Goiás e PEN1–PEN4.

O [índice por operação](evidencias.md) relaciona os cenários e passos
da matriz do Todos com estes registros, sem inventar provas de escrita.

## Procedimento para agentes

1. Localize o cenário pelo nome, instalação ou operação em `cassettes/`.
   Por exemplo, `rg -l 'documento_gerar' cassettes/`, ou
   `uvx --from okf-parser==0.48.2 okf-parser search cassettes 'documento'`.
2. Leia o card. Os campos `source_zip` e `archive_member`
   indicam o arquivo exato. Não monte URLs a partir do nome do cenário.
3. Baixe o ZIP do Internet Archive. Confirme seu `archive_sha1`.
4. Abra **somente** `archive_member` no ZIP (sem extrair caminhos
   desconhecidos). Compare seu SHA-256 após CRLF→LF com
   `cassette_sha256_lf`.
5. Exija `schema_version: 1`, `complete: true` e origem numa das
   sete instalações permitidas. Faça replay **offline**.
6. Distinga o que a resposta mostrou daquilo que seria necessário
   provar sobre uma operação completa.

## Caminho curto para qualquer agente — mesmo sem rede no sandbox

**Cassette-first, não cassette-gated:** localize o card antes de inventar
uma fixture; se não houver observação aplicável, siga com hipótese e teste.
Não confunda falha de rede do sandbox com falta de cassette: use o
[workflow opcional de inspeção](../.github/workflows/inspecionar-cassettes.yml).
Ele baixa ZIPs públicos, verifica SHA-1/SHA-256, `schema_version`,
`complete` e contagem, e mostra apenas métodos, nomes de ações e
estrutura das requisições, nunca HTML, cookies ou URLs assinadas.

**Com GitHub CLI** (depois da integração do workflow na `main`):

```sh
gh workflow run inspecionar-cassettes.yml --ref main \
  -f cards="cassettes/unb/documento-escritas-unb--36a904749e20.md"
gh run list --workflow inspecionar-cassettes.yml --limit 3
# Abra o run indicado para conferir PASS/FAIL e os logs sanitizados.
```

**Só com o conector GitHub, sem `gh` nem acesso de rede local:**
na branch da PR ativa, crie temporariamente
`.github/cassettes-a-inspecionar.txt`, com **um caminho de card por
linha** (em `cassettes/`). O evento `pull_request` executará a mesma
inspeção no Actions. Consulte o run; depois **remova o arquivo da PR**
para não persistir solicitações transitórias. Não é necessário escrever
outro workflow. O workflow não é gate do CI regular.

**Com rede local:** rode
`python3 scripts/inspecionar_cassettes.py cassettes/<instalacao>/<card>.md`
ou forneça vários cards. O script usa apenas a biblioteca padrão Python.

A inspeção **não é replay ou prova de paridade**. Na mesma fatia,
importe as observações pertinentes ao estado do anti-IES, compare
separadamente as respostas originais com o leitor IES e com o simulador,
e execute o ciclo completo da CLI. Onde a captura não mostrar corpos de
POST ou transições, use hipóteses testáveis sem bloquear o progresso.
Ausência de cassette no runner: registre o motivo e continue.

## Exemplo verificado: bloco no PEN2

```sh
curl -fL -o /tmp/pen2.zip \
  https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen2.zip
echo '32cd3f7feb5a4a80b3b363e9750b9ce0b593d2ff  /tmp/pen2.zip' | sha1sum -c -
python3 - <<'PY'
import hashlib, json, zipfile
with zipfile.ZipFile('/tmp/pen2.zip') as z:
    raw = z.read('bloco-ciclo-pen2.json')
assert hashlib.sha256(raw.replace(b'\r\n', b'\n')).hexdigest() == (
    '1b3b82ded33b541ecc9396edfad6ae5a6137073d0a02168522e330d2fd0ef677'
)
cassette = json.loads(raw)
assert cassette['schema_version'] == 1 and cassette['complete'] is True
print(len(cassette['interactions']), 'interações registradas')
PY
```

## Replay do acervo

Não há `ies replay`. O replay das cassettes é a suíte de testes
`#[ignore]` dos módulos `*::acervo*` do crate: cada teste confere os cards
(SHA-1 do ZIP, SHA-256 do membro, schema, host) e passa as respostas
gravadas pelos **mesmos leitores** de um comando, offline, imprimindo uma
tabela de lido, recusado e não aplicável. O binário de produção não lê
cassettes ([decisão](decisoes/acervo/replay-fora-da-cli.md)).

O [script `scripts/replay`](../scripts/replay) (POSIX sh, `-h` para ajuda)
é a porta de entrada:

```sh
scripts/replay                     # lista os testes do acervo
scripts/replay acervo              # roda todos, um por vez
scripts/replay processo_obter::acervo_versoes
IES_ACERVO_SAIDA=saida.md scripts/replay replay_de_unidade_trocar
```

Com filtro, ele executa
`cargo test --bin ies -- --ignored --nocapture --test-threads=1 FILTRO`.
Os ZIPs ficam em `~/.cache/ies`, ou na pasta de `IES_ACERVO_CACHE`, com o
nome do fim de `source_zip`; baixe-os e confira-os como no exemplo acima.
No ciclo anti-IES, o ZIP conferido é a fonte durável; o cache do
GitHub Actions só acelera a obtenção e o SQLite de testes é reconstruído.
O [esquema evolutivo e ciclo fechado](anti-ies/ciclo-cassettes.md)
permitem que a mesma PR altere parser, tipos, SQL e servidor.
Falta de ZIP é pânico com o nome do arquivo; o script não baixa nada.
`IES_ACERVO_SAIDA` grava o relatório num arquivo em vez de imprimi-lo
(cada teste sobrescreve o arquivo; use com o filtro de um só teste).
A rodada inteira (onze testes) levou cerca de dez minutos em 05/11/2026.

Os relatórios de cada rodada ficam em `docs/validacao-replay/`.

## Limites da evidência

O corpus histórico preserva metadados da requisição e respostas
observadas, **mas não todos os corpos das requisições POST**.
O replay pode demonstrar layouts, paginação, HTML, mensagens e
respostas; ele **não prova sozinho paridade de escrita**.
HTTP 200 não significa efeito realizado. Para escritas, exija
conferência posterior do estado, em cenário testável.

Jamais grave ou publique capturas de produção, credenciais, cookies,
URLs assinadas, dados pessoais ou HTML autenticado no Git ou em logs.
Um cenário novo gera novo card e, se necessário, novo ZIP imutável
no Archive. Não substitua a captura histórica.

Leia também [como o SEI funciona](como-o-sei-funciona/README.md)
e o [contrato da CLI](../contracts/cli.md).

## E2E como teste de aceitação (11/11/2026)

Os antigos cenários BDD/Gherkin/Cucumber foram aposentados. A referência
para qualquer nova alteração Web é a **cassette original verificável**:
primeiro os cards e hashes do acervo; depois o binário real do IES
sobre a resposta gravada e, para transição, CLI ↔ anti-IES ↔ SQLite
compartilhado semeado com fatos da cassette. O simulador deve ser comparado
**independentemente** com a resposta original, nunca apenas com o parser
que ele próprio alimentou. Captura sem POST ou releitura mantém essa
transição como hipótese; registre a lacuna e avance com fixture mínima
rotulada, sem confundir teste sintético com replay.

O [ciclo de cassettes](anti-ies/ciclo-cassettes.md) e a
skill de E2E `.agents/skills/cassette-e2e/SKILL.md` são normativos.
Não apagar testes Rust úteis que ainda protejam invariantes não gravadas
sem antes substituí-los por cobertura equivalente.
