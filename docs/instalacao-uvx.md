---
type: Guide
title: Executar IES Rust pelo uvx
description: Distribuicao de wheels nativos via maturin sem PyO3 nem Cargo para usuarios
---

# Executar o IES pelo uvx

O IES continua sendo uma **CLI compilada em Rust**. O `maturin`
empacota o executavel `ies` como script nativo do wheel Python, em
`bindings = "bin"`. Nao ha modulo Python, wrapper, chamada subprocess
nem dependencia do interpretador Python durante a execucao do IES.

O nome de distribuicao Python e **`ies-sei`**, porque `ies` ja esta
ocupado por outro projeto no PyPI. O executavel continua se chamando
**`ies`**.

## Executar sem instalar Rust (apos publicar os wheels)

```sh
uvx ies-sei --help
uvx ies-sei --version
uvx ies-sei status --json
```

O `uvx` prepara um ambiente isolado, baixa o wheel compatível com o
sistema operacional e coloca o executavel Rust no caminho de execucao.
O wheel **nao compila Rust na maquina do usuario**. As credenciais e
configuracoes privadas permanecem nos arquivos privados ja usados pelo
IES; o empacotamento nao altera as fronteiras de seguranca.

Alternativamente, para instalar uma unica vez e chamar `ies`
diretamente:

```sh
uv tool install ies-sei
ies --help
sei --help
ies-sei --help
```

**Atencao:** estes comandos do indice PyPI so funcionam **depois** de
publicar uma versao de `ies-sei` com wheels para a plataforma do
usuario. Um wheel ausente pode forcar compilacao de fonte ou impedir a
instalacao. Esta PR nao publica nada no PyPI.

## Testar antes de publicar

O workflow `.github/workflows/wheel-uvx.yml` gera wheels nativos no
Linux x86_64, macOS x86_64 e Windows x86_64. Em cada sistema, testa
`uvx --from <wheel> ies --version` e `--help`, sem acesso ao SEI.
Os arquivos produzidos ficam apenas como artifacts temporarios do
GitHub Actions, **nao no Git**. Para testar um wheel baixado:

```sh
uvx --from ./dist/ies_sei-0.1.0-<tags-da-plataforma>.whl ies --help
```

O nome acima e ilustrativo: use o caminho do wheel real baixado do CI.

Para compilar a partir do repositorio localmente, quem **desenvolve**
precisa ter Rust/Cargo instalado:

```sh
uvx --from 'git+https://github.com/franklinbaldo/ies' ies --help
```

Essa ultima forma instala a fonte Git, portanto necessita uma compilacao
Rust no primeiro uso. **Nao** confundir com executar wheels pre-compilados.

## GitHub Releases: instalar sem PyPI

Os wheels validados no CI podem ser anexados como assets de uma
GitHub Release. **O uvx aceita a URL HTTPS direta de um arquivo .whl**,
sem compilar Rust e sem consultar o PyPI:

```sh
uvx --from 'URL_HTTPS_DO_WHEEL_DA_RELEASE' ies --help
uv tool install 'URL_HTTPS_DO_WHEEL_DA_RELEASE'
```

O segundo comando instala o executavel permanentemente com o nome
`ies`. A URL precisa apontar ao **arquivo .whl**, nao a pagina
HTML da release e nem a um executavel `ies` avulso.

Para um futuro asset Linux de versao 0.1.0, por exemplo, o endereco
poderia ter esta forma (exemplo, **nao indica uma Release existente**):

```text
https://github.com/franklinbaldo/ies/releases/download/v0.1.0/ies_sei-0.1.0-py3-none-manylinux_2_34_x86_64.whl
```

Cada plataforma tem seu proprio wheel. O `uvx --from URL` **nao
seleciona automaticamente entre arquivos diferentes de uma Release**;
e necessario usar a URL cujo nome corresponda ao sistema e arquitetura.
Para selecao automatica com um unico nome de pacote, precisa de um
indice Python PEP 503 (PyPI ou indice proprio, por exemplo GitHub Pages).

### Versionamento semantico (SemVer)

O IES mantem o formato SemVer **0.MINOR.PATCH**: `patch` corrige sem
alterar a interface; `minor` acrescenta capacidade ou altera o contrato
publico, inclusive com quebra de compatibilidade. `major` e proibido
por enquanto, evitando uma promocao acidental para 1.0.0. Enquanto estiver em
`0.x.y`, a CLI ainda esta em desenvolvimento; uma quebra planejada
pode ser publicada como nova `minor`, se documentada e aprovada.

Para **preparar** uma nova versao, sem publicar nada:

```sh
python3 -I scripts/versao_semver.py patch
python3 -I scripts/versao_semver.py minor
```

O comando faz **previsao por padrao**. Depois de conferir o tipo correto:

```sh
python3 -I scripts/versao_semver.py minor --aplicar
git diff -- Cargo.toml pyproject.toml Cargo.lock
```

`--aplicar` atualiza de forma coordenada somente as declaracoes de
versao do **executavel raiz `ies`** em `Cargo.toml`,
`pyproject.toml` e `Cargo.lock`. As versoes das dependencias e dos
crates internos sao preservadas. Se a versao inicial divergir entre
os tres arquivos, a operacao recusa antes de gravar.

### Check de bump no CI

O hook canônico `semver-bump` do `prek` roda em todo CI de PR e
push na `main`. Ele falha se:

- qualquer um dos manifestos `Cargo.toml`, `pyproject.toml` e
  `Cargo.lock` nao tiver a mesma versao do executavel;
- a versao nao for `0.MINOR.PATCH` simples (sem `-rc` ou `1.0.0`);
- um PR ou push na `main` diminuir a versao em relacao ao primeiro
  pai do commit que o GitHub Actions esta validando.

O hook **nao exige bump em todas as PRs**: o IES integra muitas fatias
de trabalho em paralelo, e cada mudanca isolada nao precisa gerar
outra release. Quando um bump for solicitado, a versao deve subir,
nunca recuar. O workflow de release por tag exige igualdade exata
entre tag e manifestos; uma publicacao nao e feita num commit comum.

O comando `uvx --from ...` na Release usa o numero da versao
conferida pelo gate.

O incremento deve entrar numa **PR revisada**, com os gates Rust e wheel
verdes, antes de criar uma tag. O binario exibe a versao do Cargo pela
flag `ies --version`; o nome da wheel e a tag refletem a mesma versao.
O verificador da release tambem confere `Cargo.lock` para impedir
publicacao de pacote inconsistente com `cargo --locked`.

Por ora, a automacao de releases aceita somente versoes finais
`vX.Y.Z`; pre-releases `-rc.1`, `-beta.1` exigem convencao explicita
para a representacao de versao Python/PEP 440, e nao devem ser publicadas
sem essa etapa. O incremento **nao faz commit, nao cria tag e nao publica**
nada por conta propria.

### Protocolo de publicacao

`.github/workflows/release-uvx.yml` roda somente quando uma tag
`vMAJOR.MINOR.PATCH` e enviada ao GitHub. Antes de criar a Release:

1. Confere que a tag identifica um commit integrado a `main`.
2. Confere igualdade da versao de `Cargo.toml`, `pyproject.toml` e tag.
3. Roda todos os gates canonicos do `prek` sobre o commit da tag.
4. Reutiliza o workflow de wheels para Linux, Windows e macOS; cada um
   executa o CLI real com `uvx --from <wheel> ies --help`.
5. So apos tres wheels aprovados, cria uma GitHub Release com os tres
   arquivos `.whl` e um `SHA256SUMS`, com permissao de escrita limitada
   ao job final.

Para publicar, apos revisar e integrar a PR correspondente, a pessoa
responsavel cria e envia **explicitamente** uma tag a partir da `main`.
Por exemplo, quando as versoes no codigo forem `0.1.0`:

```sh
git switch main
git pull --ff-only
git tag -a v0.1.0 -m "IES v0.1.0"
git push origin v0.1.0
```

**Esta PR nao cria tag nem publica Release automaticamente.** Nao
versione os binarios no Git; eles ficam apenas no armazenamento de
releases. Verifique os hashes antes de redistribuir os arquivos.

## Opcao futura: indice Python

Para permitir o comando mais curto, sem URL especifica da arquitetura,

```sh
uvx --from ies-sei ies --help
```

e necessario publicar `ies-sei` no PyPI ou informar ao `uvx` um
indice alternativo (PEP 503) que liste os wheels da Release. Em ambos
os casos o `uvx` escolhe o wheel compatível automaticamente.

A publicacao no PyPI permanece uma etapa **independente e opcional**.

Antes de uma release:

1. Conferir que `Cargo.toml` e `pyproject.toml` tem a mesma versao.
2. Confirmar os gates canonicos (`prek`) e os tres smoke tests de wheel.
3. Configurar um projeto `ies-sei` no PyPI com Trusted Publishing
   autorizado ao repositorio e workflow de release revisado.
4. Somente numa release aprovada, publicar os wheels por OIDC e verificar
   o comando `uvx --from ies-sei ies --version` no indice real.

A PR de empacotamento **nao** cria automatismo de upload nem tokens.
Lembre que rodas com OpenSSL ou glibc podem exigir verificacao adicional
de compatibilidade (manylinux) antes da publicacao.
