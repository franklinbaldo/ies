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
uvx --from ies-sei ies --help
uvx --from ies-sei ies --version
uvx --from ies-sei ies status --json
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

## Publicacao

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
